use windivert::{
    layer::NetworkLayer, packet::WinDivertPacket, prelude::WinDivertFlags, CloseAction, WinDivert};
use pnet::packet::{icmpv6::{ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket}, Icmpv6Packet, Icmpv6Types::RouterAdvert}, ipv6::Ipv6Packet, Packet};
use ipnet::Ipv6Net;
use log::{info,debug,error,warn};
use std::{sync::{atomic::Ordering,Arc}, vec,collections::VecDeque};
use tokio::sync::{mpsc,RwLock,Mutex};
use crate:: utils::ipv6_addr_u8_to_string;
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use crate::globals::{get_container_data, BLACKLIST_MODE,get_interface_name};

const BUFFER_SIZE: usize = 1500;  // 每个数据包的缓冲区大小
const POOL_SIZE: usize = 10;  // 内存池中缓冲区的数量

#[derive(Clone)]
struct MemoryPool {
    pool: Arc<Mutex<VecDeque<Vec<u8>>>>,
}
impl MemoryPool {
    fn new() -> Self {
        let mut pool = VecDeque::with_capacity(POOL_SIZE);
        // 初始化内存池，填充一些缓冲区
        for _ in 0..POOL_SIZE {
            pool.push_back(vec![0u8; BUFFER_SIZE]);
        }
        MemoryPool {
            pool: Arc::new(Mutex::new(pool)),
        }
    }
    // 从池中获取一个缓冲区
    async fn acquire(&self) -> Vec<u8> {
        let mut pool = self.pool.lock().await;
        if let Some(buffer) = pool.pop_front() {
            buffer // 如果有缓冲区，直接返回
        } else {
            vec![0u8; BUFFER_SIZE] // 如果池中没有缓冲区，则创建一个新的缓冲区
        }
    }
    // 将缓冲区归还到池中
    async fn release(&self, buffer: Vec<u8>) {
        let mut pool = self.pool.lock().await;
        if pool.len() < POOL_SIZE {
            pool.push_back(buffer); // 如果池还没有满，将缓冲区归还池中
        }
    }
}
pub async fn wdvt_process(){
    let wdvt={
        let interface_name=get_interface_name();
        let filter=match interface_name{
            Some(name) => 
                {   
                    //format!("inbound and !loopback and ipv6.InterfaceName==\"{}\" and icmpv6.Type==134",name)
                    format!("inbound and !loopback and ifidx=={} and icmpv6.Type==134",name.index)
            },
            None => 
                format!("inbound and !loopback and icmpv6.Type==134"),
        };
        //let filter="inbound and !loopback and icmpv6.Type==134";
        let flags=WinDivertFlags::new();
        let wdvt=WinDivert::network(filter, 1,flags).unwrap();
        Arc::new(RwLock::new(wdvt))
    };
    let condition=|data: &[u8]| -> bool {
        let ipv6_prefix: Vec<Ipv6Net> = get_container_data();
        let ipv6_packet = match Ipv6Packet::new(data) {
            Some(packet) => {
                debug!("Recived an IPv6 packet!");
                packet
            }
            None => return true,
        };
        // 尝试解析 ICMPv6 包
        let icmp6_packet = match Icmpv6Packet::new(ipv6_packet.payload()) {
            Some(packet) => {
                debug!("It's an ICMPv6 packet!");
                packet
            }
            None => return true,
        };
        // 判断是否为 Router Advertisement
        if icmp6_packet.get_icmpv6_type() != RouterAdvert {
            debug!("It's not a RouterAdvert packet!");
            return true;
        }
        // 获取 RouterAdvertPacket 对象
        let ra_packet = match RouterAdvertPacket::new(icmp6_packet.packet()) {
            Some(packet) => {debug!("Received an ICMPv6 Router Advertisement!"); packet},
            None => return true,
        };
        // 遍历 RouterAdvertPacket 的 options
        for op in ra_packet.get_options() {
            if op.option_type != PrefixInformation {
                // 非携带 PrefixInformation， 跳过
                continue;
            }
    
            // 获取 PrefixInformationPacket 对象（是一个NdpOption）
            let option_raw = op.to_bytes();
            let pfi = match PrefixInformationPacket::new(&option_raw) {
                Some(packet) => {
                    debug!("Find PrefixInformationPacket ndp option!");
                    packet
                }
                None => continue,
            };
    
            // 获取 数据包中的 IPv6 前缀（字符串）
            let pkt_prefix_str = ipv6_addr_u8_to_string(pfi.payload());
            info!("Recived an IPv6 Prefix: {}", pkt_prefix_str);
    
            let blacklist_mode =  BLACKLIST_MODE.load(Ordering::SeqCst) ;
    
            let is_prefix_in_list = ipv6_prefix
                .iter()
                .any(|&prefix| prefix.addr().octets() == pfi.payload());
    
            let verdict = match (blacklist_mode, is_prefix_in_list) {
                (false, true)|(true, false) => {info!("Accepted prefix {}!", pkt_prefix_str); true},
                _ => {info!("Dropped! prefix {}!", pkt_prefix_str); false},
            };
            return verdict;
        }
        debug!("No PrefixInformation was found in packet.");
        return true;
    };
    let (tx,rx)=mpsc::channel(100);
    let wdvt_clone=Arc::clone(&wdvt);
    let _handle=tokio::spawn(async move {
        recv_packet(wdvt_clone, tx).await;
    });
    process_packet(wdvt, condition,rx).await;
    
    //handle.abort();
    info!("wdvt process stopped!");
}
//这是一个异步函数，用于接收数据包，并将其发送到指定的 channel 中。
async fn recv_packet(
    wdvt_rwlock: Arc<RwLock<WinDivert<NetworkLayer>>>,
    tx: mpsc::Sender<WinDivertPacket<'static, NetworkLayer>>,
) {    
    let memory_pool = MemoryPool::new();
    loop {
        //let mut packet_buffer = vec![0u8; 1500]; // 独立缓冲区
        // 从内存池获取缓冲区
        let mut packet_buffer = memory_pool.acquire().await;
        let wdvt=wdvt_rwlock.read().await;
        let packet_result = wdvt.recv(Some(&mut packet_buffer)); // 从 wdvt 接收数据

        match packet_result {
            Ok(packet) => {
                // 使用 Clone 方法创建一个独立的 WinDivertPacket
                let owned_packet = packet.into_owned();

                // 发送克隆后的数据包
                if tx.send(owned_packet).await.is_err() {
                    error!("Send packet to process failed!");
                    break;
                }
            }
            Err(e) => {
                warn!("Recv error: {}", e);
                break;
            }
        }
        // 将缓冲区归还到内存池中
        memory_pool.release(packet_buffer).await;
    }
}

//这是一个异步函数，用于处理数据包，并根据条件决定是否发送到另一个网络接口。
// condition 是一个函数，用于判断是否应该处理该数据包。
// rx 是一个 channel，用于接收数据包。
async fn process_packet<F>(wdvt_rwlock: Arc<RwLock<WinDivert<NetworkLayer>>>,
     condition: F,mut rx: mpsc::Receiver<WinDivertPacket<'static,NetworkLayer>>)
where
    F: Fn(&[u8]) -> bool + Send + 'static,
{   
    let mut running = true;
    loop {
        tokio::select! {
            _=tokio::signal::ctrl_c() => {
                info!("Ctrl+C received, stopping...");
                running = false;
            },
            result=rx.recv() => {
                match result {
                    Some(packet) => {
                        let data=packet.data.as_ref();
                        if condition(data) {
                            let wdvt = wdvt_rwlock.read().await;
                            send_twice(&*wdvt, &packet);
                        }else {
                            debug!("Drop packet!");
                        }
                    },
                    None => {
                        error!("Recv error: channel closed!");
                        break;
                    }
                }
            },
        }
        if !running {
            let mut wdvt = wdvt_rwlock.write().await;
            match wdvt.close(CloseAction::Nothing){
                Ok(_) => {
                    info!("Close wdvt successfully!");
                },
                Err(e) => {
                    error!("Close wdvt failed: {}", e);
                }
            }
            break;
        }
    }
}
// 发送数据包两次，防止丢包。
fn send_twice(wdt:&WinDivert<NetworkLayer>,packet:&WinDivertPacket<NetworkLayer>){
    for _ in 0..2 {
        if wdt.send(packet).is_ok() {
            debug!("Packet sent successfully!");
            return;
        }
    }
    warn!("Send twice failed!");
}
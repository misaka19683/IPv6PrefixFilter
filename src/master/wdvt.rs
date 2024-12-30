use windivert::{
    layer::NetworkLayer, packet::WinDivertPacket, prelude::WinDivertFlags, WinDivert};
use pnet::packet::{ Packet,ipv6::Ipv6Packet,
    icmpv6::{Icmpv6Types::RouterAdvert,Icmpv6Packet,
    ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket}}};
use ipnet::Ipv6Net;
use log::{info,debug,error};
use std::sync::{atomic::Ordering,Arc};
use tokio::sync::mpsc;
use crate:: utils::ipv6_addr_u8_to_string;
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use crate::globals::{get_container_data, BLACKLIST_MODE};

pub async fn wdvt_process(){
    let filter="inbound and !loopback and icmpv6.Type==134";
    let wdvt=Arc::new(WinDivert::network(filter, 1,WinDivertFlags::new()).unwrap());
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
    tokio::spawn(async move {
        recv_packet(wdvt_clone, tx).await;
    });
    process_packet(wdvt, condition,rx).await;
}
//这是一个异步函数，用于接收数据包，并将其发送到指定的 channel 中。
async fn recv_packet(wdvt:Arc<WinDivert<NetworkLayer>>,tx: mpsc::Sender<WinDivertPacket<'static,NetworkLayer>>) {
    loop {
        match wdvt.recv(None) {
            Ok(packet) => {
                if tx.send(packet).await.is_err() {
                    error!("Send packet failed!");
                    break;
                }
            },
            Err(e)=>{
                error!("Recv error: {}", e);
                break;
            },
        }
    }
}
//这是一个异步函数，用于处理数据包，并根据条件决定是否发送到另一个网络接口。
// condition 是一个函数，用于判断是否应该处理该数据包。
// rx 是一个 channel，用于接收数据包。
async fn process_packet<F>(wdvt: Arc<WinDivert<NetworkLayer>>, condition: F,mut rx: mpsc::Receiver<WinDivertPacket<'static,NetworkLayer>>)
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
            break;
        }
    }
}
// 发送数据包两次，防止丢包。
fn send_twice(wdt:&WinDivert<NetworkLayer>,packet:&WinDivertPacket<NetworkLayer>){
    match wdt.send(&packet) {
        Ok(_) => debug!("Send once successfully!"),
        Err(_) => {
            match wdt.send(packet) {
                Ok(_) => debug!("Send twice successfully!"),
                Err(e) => error!("Send twice failed!,{}",e),
            }
        }
    }
}
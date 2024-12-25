use ipnet::Ipv6Net;
use log::{debug, info};
#[cfg(target_os = "linux")]
use nfq::{Queue, Verdict};
use pnet::packet::{
    icmpv6::{
        ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket},
        Icmpv6Packet,
        Icmpv6Types::RouterAdvert,
    },
    ipv6::Ipv6Packet,
    Packet,
};
use std::sync::atomic::Ordering;

use crate::error::AppError;
use crate::globals::{get_container_data, BLACKLIST_MODE, QUEUE_NUM};
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use crate::utils::ipv6_addr_u8_to_string;
#[cfg(target_os = "linux")]
/// 启动队列监听器
pub fn start_queue() -> std::result::Result<Queue, AppError> {
    let mut queue = Queue::open().map_err(|e| AppError::QueueStartError(e.to_string()))?; // 打开 NFQUEUE
    queue
        .bind(QUEUE_NUM)
        .map_err(|e| AppError::QueueStartError(e.to_string()))?; // 绑定到队列 0
    queue
        .set_fail_open(QUEUE_NUM, false)
        .map_err(|e| AppError::QueueStartError(e.to_string()))?; // 队列满时拒绝数据包
    Ok(queue)
}
#[cfg(target_os = "linux")]
pub fn process_queue() {
    use tokio::{runtime::Runtime,  task};
    let mut queue = match Queue::open() {
        Ok(q) => q,
        Err(err) => {
            eprintln!("无法打开 NFQUEUE: {}", err);
            return;
        }
    };
    if let Err(err) = queue.bind(QUEUE_NUM) {
        eprintln!("无法绑定到队列 {}: {}", QUEUE_NUM, err);
        return;
    }
    if let Err(err) = queue.set_fail_open(QUEUE_NUM, false) {
        eprintln!("无法设置 fail-open 行为: {}", err);
        return;
    }
    let rt=Runtime::new().unwrap();
    let handle=task::spawn_blocking(move||{
        loop {
            match queue.recv() {
                Ok(mut msg) => {
                    let data = msg.get_payload();
                    let verdict = handle_packet(data);
                    msg.set_verdict(verdict);
                    queue.verdict(msg).unwrap();
                }
                Err(_) => {
                    eprint!("NFQUEUE 接收数据包失败！");
                    //continue;
                    break;
                }
            }
        }   
    });
    rt.block_on(async{
        let _=tokio::signal::ctrl_c().await;
        println!("接收到SIGINT，正在优雅地关闭程序...");
        handle.abort();
        handle.await.unwrap();
        info!("Queue listener stopped!");
    });
}
    // while stop_flag.load(Ordering::SeqCst) {
    //     match queue.recv() {
    //         Ok(mut msg) => {
    //             let data = msg.get_payload();

    //             let verdict = handle_packet(data);

    //             msg.set_verdict(verdict);

    //             queue.verdict(msg).unwrap();
    //         }
    //         Err(_) => {
    //             sleep(Duration::from_millis(50));
    //             continue;
    //         }
    //     }
    // }
#[cfg(target_os = "linux")]
/// 处理数据包
fn handle_packet(data: &[u8]) -> Verdict {
    //获取全局变量ipv6_prefix
    let ipv6_prefix: Vec<Ipv6Net> = get_container_data();
    // 尝试解析 IPv6 包
    let ipv6_packet = match Ipv6Packet::new(data) {
        Some(packet) => {
            debug!("Recived an IPv6 packet!");
            packet
        }
        None => return Verdict::Accept,
    };

    // 尝试解析 ICMPv6 包
    let icmp6_packet = match Icmpv6Packet::new(ipv6_packet.payload()) {
        Some(packet) => {
            debug!("It's an ICMPv6 packet!");
            packet
        }
        None => return Verdict::Accept,
    };

    // 判断是否为 Router Advertisement
    if icmp6_packet.get_icmpv6_type() != RouterAdvert {
        debug!("It's not a RouterAdvert packet!");
        return Verdict::Accept;
    }

    // 获取 RouterAdvertPacket 对象
    let ra_packet = match RouterAdvertPacket::new(icmp6_packet.packet()) {
        Some(packet) => {debug!("Received an ICMPv6 Router Advertisement!"); packet},
        None => return Verdict::Accept,
    };

    // 遍历 RA Packet 中的 NdpOption 并查找包含 Prefix Information 的 NdpOption
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
            (false, true) => {info!("Accepted prefix {}!", pkt_prefix_str); Verdict::Accept},
            (true, false) => {info!("Accepted prefix {}!", pkt_prefix_str); Verdict::Accept},
            _ => {info!("Dropped! prefix {}!", pkt_prefix_str); Verdict::Drop},
        };
        return verdict;
    }
    debug!("No PrefixInformation was found in packet.");
    return Verdict::Accept;
}

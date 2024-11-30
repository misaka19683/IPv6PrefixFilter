use log::{info, debug};
use nfq::{Queue, Verdict};
use pnet::packet::{Packet, icmpv6::Icmpv6Packet, ipv6::Ipv6Packet};
use pnet::packet::icmpv6::ndp::RouterAdvertPacket;
use pnet::packet::icmpv6::ndp::NdpOptionTypes::PrefixInformation;
use pnet::packet::icmpv6::Icmpv6Types::RouterAdvert;
//use std::sync::atomic::{AtomicBool, Ordering};
//use std::io::Result;

// use crate::order_parser::get_prefix;
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use crate::utils::ipv6_addr_u8_to_string;

/// 启动队列监听器
pub fn start_queue(queue_num: u16) -> std::io::Result<Queue> {
    let mut queue = Queue::open()?; // 打开 NFQUEUE
    queue.bind(queue_num)?; // 绑定到队列 0
    queue.set_fail_open(0, false)?; // 队列满时拒绝数据包
    Ok(queue)
}


// pub fn loop_queue(running: std::sync::Arc<AtomicBool>) -> io::Result<()> {

//     while running.load(Ordering::SeqCst) {
//         while running.load(Ordering::SeqCst) {
//             if let Ok(mut msg) = queue.recv() {
//                 let data = msg.get_payload();
//                 let verdict = handle_packet(data);
//                 msg.set_verdict(verdict);
//                 queue.verdict(msg)?;
//             }
//         }
    
//         Ok(())
//     }

//     Ok(())
// }
pub fn process_queue(queue: &mut Queue) -> std::io::Result<()> {
    while let Ok(mut msg) = queue.recv() {
        let data = msg.get_payload();
        let verdict = handle_packet(data);
        msg.set_verdict(verdict);
        queue.verdict(msg)?;
    }
    Ok(())
}

// 清理队列
pub fn end_queue(queue: &mut Queue,queue_num: u16) -> std::io::Result<()> {
    queue.unbind(queue_num)?;
    println!("Queue unbound. Cleanup completed.");
    Ok(())
    
}

/// 处理数据包
fn handle_packet(data: &[u8]) -> Verdict {
    // let ipv6_prefix = get_prefix();
    let ipv6_prefix = [];
    let ipv6_prefix = &ipv6_prefix;
    let ipv6_prefix_str = ipv6_addr_u8_to_string(ipv6_prefix);
    debug!("IPv6 Prefix get from input: {}", ipv6_prefix_str);

    // 尝试解析 IPv6 包
    let ipv6_packet = match Ipv6Packet::new(data) {
        Some(packet) => {debug!("It's a IPv6 packet!"); packet},
        None => return Verdict::Accept,
    };

    // 尝试解析 ICMPv6 包
    let icmp6_packet = match Icmpv6Packet::new(ipv6_packet.payload()) {
        Some(packet) => {debug!("It's a ICMPv6 packet!");packet},
        None => return Verdict::Accept,
    };

    // 判断是否为 Router Advertisement
    if icmp6_packet.get_icmpv6_type() != RouterAdvert {
        debug!("It's not a RouterAdvert packet!");
        return Verdict::Accept;
    }

    debug!("Received ICMPv6 Router Advertisement!");

    let ra_packet = match RouterAdvertPacket::new(icmp6_packet.packet()) {
        Some(packet) => packet,
        None => return Verdict::Accept,
    };

    // 遍历选项并检查 Prefix Information
    for op in ra_packet.get_options() {
        if op.option_type != PrefixInformation {
            continue;
        }
        let option_raw = op.to_bytes();
        let pfi = match PrefixInformationPacket::new(&option_raw) {
            Some(packet) => {debug!("Find PrefixInformationPacket ndp option!"); packet},
            None => continue,
        };
        let pkt_prefix_str = ipv6_addr_u8_to_string(pfi.payload());
        debug!("IPv6 Prefix in packet is {}", pkt_prefix_str);
        if pfi.payload() == ipv6_prefix {
            info!("Accepted prefix {}!", pkt_prefix_str);
            return Verdict::Accept;
        } else {
            info!("Droped! prefix {}!", pkt_prefix_str);
            return Verdict::Drop;
        }
    }

    Verdict::Accept
}

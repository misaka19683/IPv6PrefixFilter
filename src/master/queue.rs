use log::{debug, info};
use nfq::{Queue, Verdict};
use std::sync::{Arc, Mutex};
use pnet::packet::{ Packet,ipv6::Ipv6Packet,
        icmpv6::{Icmpv6Types::RouterAdvert,Icmpv6Packet,
        ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket}}};

use crate::error::AppError;
use crate::globals::{get_container_data, QUEUE_NUM};
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use crate::utils::ipv6_addr_u8_to_string;

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

pub fn process_queue(
    queue: &mut Queue,
    stop_flag: Arc<Mutex<bool>>,
) -> std::result::Result<(), AppError> {
    while !*stop_flag.lock().unwrap() {
        match queue.recv() {
            Ok(mut msg) => {
                let data = msg.get_payload();

                let verdict = handle_packet(data);

                msg.set_verdict(verdict);

                queue.verdict(msg)?;
            }
            Err(_) => {
                return Err(AppError::QueueProcessError(
                    "Failed to receive packet from queue".to_string(),
                ));
            }
        }
    }
    Err(AppError::Interrupt)
}

/// 处理数据包
fn handle_packet(data: &[u8]) -> Verdict {
    //获取全局变量ipv6_prefix
    let ipv6_prefix = get_container_data();
    // 尝试解析 IPv6 包
    let ipv6_packet = match Ipv6Packet::new(data) {
        Some(packet) => {
            debug!("It's a IPv6 packet!");
            packet
        }
        None => return Verdict::Accept,
    };

    // 尝试解析 ICMPv6 包
    let icmp6_packet = match Icmpv6Packet::new(ipv6_packet.payload()) {
        Some(packet) => {
            debug!("It's a ICMPv6 packet!");
            packet
        }
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
            Some(packet) => {
                debug!("Find PrefixInformationPacket ndp option!");
                packet
            }
            None => continue,
        };
        let pkt_prefix_str = ipv6_addr_u8_to_string(pfi.payload());
        debug!("IPv6 Prefix in packet is {}", pkt_prefix_str);
        // if pfi.payload() == ipv6_prefix {
        //     info!("Accepted prefix {}!", pkt_prefix_str);
        //     return Verdict::Accept;
        if ipv6_prefix
            .iter()
            .any(|&prefix| prefix.octets() == pfi.payload())
        {
            info!("Accepted prefix {}!", pkt_prefix_str);
            return Verdict::Accept;
        } else {
            info!("Droped! prefix {}!", pkt_prefix_str);
            return Verdict::Drop;
        }
    }
    Verdict::Accept
}

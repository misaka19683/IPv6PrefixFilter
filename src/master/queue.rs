use log::{debug, info};
use ipnet::Ipv6Net;
use nfq::{Queue, Verdict};
use std::{sync::{Arc, Mutex}, thread::sleep, time::Duration};
use pnet::packet::{ Packet,ipv6::Ipv6Packet,
        icmpv6::{Icmpv6Types::RouterAdvert,Icmpv6Packet,
        ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket}}};

use crate::error::AppError;
use crate::globals::{get_container_data, QUEUE_NUM, BLACKLIST_MODE};
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

pub fn process_queue(queue: &mut Queue, stop_flag: Arc<Mutex<bool>>,) 
    -> std::result::Result<(), AppError> {

    // 设置队列为非阻塞
    queue.set_nonblocking(true);
    while *stop_flag.lock().unwrap() {
        match queue.recv() {
            Ok(mut msg) => {
                let data = msg.get_payload();

                let verdict = handle_packet(data);

                msg.set_verdict(verdict);

                queue.verdict(msg)?;
            }
            Err(_) => {
                sleep(Duration::from_millis(50));
                continue;
            }
        }
    }
    Err(AppError::Interrupt)
}

/// 处理数据包
fn handle_packet(data: &[u8]) -> Verdict {
    //获取全局变量ipv6_prefix
    let ipv6_prefix:Vec<Ipv6Net> = get_container_data();
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
       
        let blacklist_mode = match BLACKLIST_MODE.lock() {//读取全局变量-黑名单模式
            Ok(guard)=> *guard,
            Err(e)=>{
                eprint!("Failed to acquire lock for BLACKLIST_MODE: {}", e);
                false
            },
        };
        let is_prefix_in_list = ipv6_prefix.iter().any(|&prefix| prefix.addr().octets() == pfi.payload());
        let verdict = decide_verdict(blacklist_mode,is_prefix_in_list);
        log_and_return(verdict, &pkt_prefix_str);
        return verdict;
    }
    Verdict::Accept
}
fn decide_verdict(blacklist_mode: bool, is_prefix_in_list: bool) -> Verdict {
        match (blacklist_mode, is_prefix_in_list) {
            (false, true) => Verdict::Accept,
            (true, false) => Verdict::Accept,
            _ => Verdict::Drop,
        }
}
fn log_and_return(verdict: Verdict, prefix: &str) {
    match verdict {
        Verdict::Accept => info!("Accepted prefix {}!", prefix),
        Verdict::Drop   => info!("Dropped! prefix {}!", prefix),
        _=>{},
    }
}
 // if pfi.payload() == ipv6_prefix {
        //     info!("Accepted prefix {}!", pkt_prefix_str);
        //     return Verdict::Accept;

        // if ipv6_prefix
        //     .iter()
        //     .any(|&prefix| prefix.addr().octets() == pfi.payload())
        // {
        //     info!("Accepted prefix {}!", pkt_prefix_str);
        //     return Verdict::Accept;
        // } else {
        //     info!("Droped! prefix {}!", pkt_prefix_str);
        //     return Verdict::Drop;
        // }
        //let blacklist_mode:bool = *BLACKLIST_MODE.lock().unwrap();
        //let blacklist_mode:bool;
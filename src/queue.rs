use log::info;
use nfq::{Queue, Verdict};
use nftables::expr::PayloadField;
use pnet::packet::{Packet, icmpv6::Icmpv6Packet, ipv6::Ipv6Packet};
use pnet::packet::icmpv6::ndp::RouterAdvertPacket;
use pnet::packet::icmpv6::ndp::NdpOptionTypes::PrefixInformation;
use pnet::packet::icmpv6::Icmpv6Types::RouterAdvert;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;
use crate::order_parser::get_prefix;

use crate::prefix_info::{PrefixInformationPacket, ToBytes};

/// 启动队列监听器
pub fn start_queue(running: std::sync::Arc<AtomicBool>) -> io::Result<()> {
    let mut queue = Queue::open()?; // 打开 NFQUEUE
    queue.bind(0)?; // 绑定到队列 0

    while running.load(Ordering::SeqCst) {
        if let Ok(mut msg) = queue.recv() {
            let data = msg.get_payload();
            let verdict = handle_packet(data);
            msg.set_verdict(verdict);
            queue.verdict(msg)?;
        }
    }

    println!("Queue stopped.");
    Ok(())
}

/// 处理数据包
fn handle_packet(data: &[u8],) -> Verdict {
    let ipv6_prefix = get_prefix();
    let ipv6_prefix = &ipv6_prefix;
    //println!("Received packet with length: {}", data.len());
    match Ipv6Packet::new(data) {
        Some(ipv6_packet) => {
            match Icmpv6Packet::new(ipv6_packet.payload()) {
                Some(icmp6_packet) => {
                    if icmp6_packet.get_icmpv6_type() == RouterAdvert {
                        info!("Received ICMPv6 Router Advertisement!");
                        let ra_packet = RouterAdvertPacket::new(icmp6_packet.packet()).unwrap();
                        for op in ra_packet.get_options() {
                            match op.option_type {
                                PrefixInformation => {
                                    match PrefixInformationPacket::new(&op.to_bytes()) {
                                        Some(pfi) => {
                                            if pfi.payload() == ipv6_prefix {
                                                return Verdict::Accept;
                                            } else {
                                                return Verdict::Drop;
                                            }
                                        },
                                        None => {return Verdict::Accept;}
                                    }
                                },
                                _ => {return Verdict::Accept;}
                            }
                        }
                        return Verdict::Accept;
                    } else {
                        return Verdict::Accept;
                    }
                }
                None => {return Verdict::Accept;},
            }
        }
        None => {return Verdict::Accept;},
    }
}

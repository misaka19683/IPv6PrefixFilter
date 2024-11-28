use nfq::{Queue, Verdict};
use pnet::packet::{Packet, icmpv6::Icmpv6Packet, ipv6::Ipv6Packet};
use pnet::packet::icmpv6::Icmpv6Types::RouterAdvert;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;

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
fn handle_packet(data: &[u8]) -> Verdict {
    //println!("Received packet with length: {}", data.len());
    match Ipv6Packet::new(data) {
        Some(ipv6_packet) => {
            match Icmpv6Packet::new(ipv6_packet.payload()) {
                Some(icmpv6_packet) => {
                    if icmpv6_packet.get_icmpv6_type() == RouterAdvert {
                        println!("Received ICMPv6 Router Advertisement!");
                        Verdict::Drop
                    } else {
                        Verdict::Accept
                    }
                }
                None => Verdict::Accept,
            }
        }
        None => Verdict::Accept,
    }
}

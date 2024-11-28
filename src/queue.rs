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
    match Ipv6Packet::new(data) {
        Some(ipv6_packet) => {
            if let Some(icmpv6_packet) = Icmpv6Packet::new(ipv6_packet.payload()) {
                if icmpv6_packet.get_icmpv6_type() == RouterAdvert {
                    println!("Dropped ICMPv6 Router Advertisement.");
                    return Verdict::Drop;
                }
            }
        }
        None => {}
    }
    Verdict::Accept
}

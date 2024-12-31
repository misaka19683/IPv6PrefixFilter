use windivert::{
    layer::NetworkLayer,
    packet::WinDivertPacket,
    prelude::WinDivertFlags,
    CloseAction, WinDivert,
};
use pnet::packet::{
    icmpv6::{ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket}, Icmpv6Packet, Icmpv6Types::RouterAdvert},
    ipv6::Ipv6Packet, Packet
};
use ipnet::Ipv6Net;
use log::{info, debug, error, warn};
use core::panic;
use std::sync::{atomic::Ordering, Arc};
use tokio::sync::{mpsc, RwLock,broadcast};
use crate::utils::ipv6_addr_u8_to_string;
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use crate::globals::{get_container_data, BLACKLIST_MODE, get_interface_name};

pub async fn wdvt_process() {
    let wdvt = {
        let interface_name = get_interface_name();
        let filter = match interface_name {
            Some(name) => format!("inbound and !loopback and ifidx=={} and icmpv6.Type==134", name.index),
            None => format!("inbound and !loopback and icmpv6.Type==134"),
        };
        let flags = WinDivertFlags::new();
        WinDivert::network(filter, 1, flags).unwrap()
    };
    let condition = |data: &[u8]| -> bool {
        let ipv6_prefix: Vec<Ipv6Net> = get_container_data();
        let ipv6_packet = match Ipv6Packet::new(data) {
            Some(packet) => packet,
            None => return true,
        };

        let icmp6_packet = match Icmpv6Packet::new(ipv6_packet.payload()) {
            Some(packet) => packet,
            None => return true,
        };

        if icmp6_packet.get_icmpv6_type() != RouterAdvert {
            return true;
        }

        let ra_packet = match RouterAdvertPacket::new(icmp6_packet.packet()) {
            Some(packet) => packet,
            None => return true,
        };

        for op in ra_packet.get_options() {
            if op.option_type != PrefixInformation {
                continue;
            }

            let option_raw = op.to_bytes();
            let pfi = match PrefixInformationPacket::new(&option_raw) {
                Some(packet) => packet,
                None => continue,
            };

            let pkt_prefix_str = ipv6_addr_u8_to_string(pfi.payload());
            info!("Received an IPv6 Prefix: {}", pkt_prefix_str);

            let blacklist_mode = BLACKLIST_MODE.load(Ordering::SeqCst);
            let is_prefix_in_list = ipv6_prefix.iter().any(|&prefix| prefix.addr().octets() == pfi.payload());

            let verdict = match (blacklist_mode, is_prefix_in_list) {
                (false, true) | (true, false) => { info!("Accepted prefix {}!", pkt_prefix_str); true },
                _ => { info!("Dropped prefix {}!", pkt_prefix_str); false },
            };
            return verdict;
        }
        true
    };
    let (tx, rx) = mpsc::channel(100);
    let (stop_tx, stop_rx1) = broadcast::channel(1);
    let stop_rx2 = stop_tx.subscribe();
    let wdvt_rwlock = Arc::new(RwLock::new(wdvt));
    let wdvt_clone = wdvt_rwlock.clone();
    tokio::spawn(async move {
        debug!("start stop signal listener");
        if let Ok(())=tokio::signal::ctrl_c().await {
            stop_tx.send(()).unwrap();
            debug!("Received Ctrl+C, and sending stop signal to wdvt process...");
        }
    });
    info!("wdvt process started!");
    tokio::spawn(async move {
        recv_packet(wdvt_rwlock, tx, stop_rx1).await;
    });
    process_packet(wdvt_clone, condition, rx, stop_rx2).await;
    //handle.await.unwrap();

    info!("wdvt process stopped!");
}

async fn recv_packet(
    wdvt_rwlock: Arc<RwLock<WinDivert<NetworkLayer>>>,
    tx: mpsc::Sender<WinDivertPacket<'static, NetworkLayer>>,
    mut stop_rx: broadcast::Receiver<()>, // 接收停止信号的通道
) {
    debug!("start recv_packet");
    loop {
        tokio::select! {
            _ = stop_rx.recv() => {
                // 收到停止信号，退出循环
                //info!("Received stop signal. Exiting...");
                debug!("recv_packet stop signal received");
                break;
            },
            // packet = tokio::task::spawn_blocking({
            //     let wdvt_clone = wdvt_rwlock.clone();
            //     move || {
            //         let mut packet_buffer = Vec::with_capacity(65500000);
            //         let wdvt = wdvt_clone.blocking_read();
            //         let data=wdvt.recv(Some(packet_buffer.as_mut_slice())).map(|data| data.into_owned());
            //         debug!("recv_packet blocking task done");
            //         data.unwrap()
            //     }
            // }) 
            packet = tokio::task::spawn_blocking({
                let wdvt_clone = wdvt_rwlock.clone();
                move || {
                    let mut packet_buffer = Vec::with_capacity(65500000000);
                    let wdvt = wdvt_clone.blocking_read();
                    match wdvt.recv(Some(packet_buffer.as_mut_slice())) {
                        Ok(data) => {
                            debug!("recv_packet blocking task done");
                            data.into_owned()
                        },
                        Err(e) => {
                            error!("Recv error: {}", e);
                            //return Err(e);
                            panic!("Recv error: {}", e);
                        }
                    }
                }
            })
            => {
                match packet {
                    Ok(packet)=> {
                        let owned_packet = packet.into_owned();
                        if tx.send(owned_packet).await.is_err() {
                            error!("Failed to send packet to processing channel!");
                            break;
                        }
                    }
                    // Ok(Err(e)) => {
                    //     warn!("Recv error: {}", e);
                    //     //break;
                    // }
                    Err(e) => {
                        error!("Failed to execute blocking task: {}", e);
                        break;
                    }
                }
            }
        }
    }
}

async fn process_packet<F>(wdvt_rwlock: Arc<RwLock<WinDivert<NetworkLayer>>>, 
    condition: F, 
    mut rx: mpsc::Receiver<WinDivertPacket<'static, NetworkLayer>>,
    mut stop_rx: broadcast::Receiver<()>, // 接收停止信号的通道
)
where
    F: Fn(&[u8]) -> bool + Send + 'static,
{
    let mut running = true;
    loop {
        tokio::select! {
            _ = stop_rx.recv() => {
                info!("Ctrl+C received, stopping...");
                running = false;
            },
            result = rx.recv() => {
                match result {
                    Some(packet) => {
                        let data = packet.data.as_ref();
                        if condition(data) {
                            let wdvt = wdvt_rwlock.read().await;
                            send_twice(&*wdvt, &packet);
                        } else {
                            debug!("Dropped packet!");
                        }
                    },
                    None => {
                        // error!("Channel closed, stopping...");
                        // break;
                        debug!("packet Channel closed")
                    }
                }
            },
        }
        if !running {
            let mut wdvt = wdvt_rwlock.write().await;
            match wdvt.close(CloseAction::Nothing) {
                Ok(_) => info!("Successfully closed wdvt!"),
                Err(e) => error!("Failed to close wdvt: {}", e),
            }
            break;
        }
    }
}

fn send_twice(wdvt: &WinDivert<NetworkLayer>, packet: &WinDivertPacket<NetworkLayer>) {
    for _ in 0..2 {
        if wdvt.send(packet).is_ok() {
            debug!("Packet sent successfully!");
            return;
        }
    }
    warn!("Send twice failed!");
}

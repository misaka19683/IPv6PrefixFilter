#[cfg(windows)]
use windivert_sys::{
    self,
    WinDivertClose, 
    WinDivertSend, 
    WinDivertFlags, 
    WinDivertLayer, 
    WinDivertOpen, 
    WinDivertRecv,
    address::WINDIVERT_ADDRESS
};
#[cfg(windows)]
use std::ffi::CString;
//use windivert_sys::address::WINDIVERT_ADDRESS;
use pnet::packet::{ Packet,ipv6::Ipv6Packet,
    icmpv6::{Icmpv6Types::RouterAdvert,Icmpv6Packet,
    ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket}}};
    use crate::globals::{get_container_data, BLACKLIST_MODE};
use ipnet::Ipv6Net;
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use std::{sync::{Arc, Mutex},thread::sleep,time::Duration};
//use crate::utils::ipv6_addr_u8_to_string;
#[cfg(windows)]
pub fn the_process(stop_flag:Arc<Mutex<bool>>)  {
    use std::thread::sleep;

    
    let filter_cstr=CString::new("icmp6.Type==134").expect("CString::new failed");
    let filter=filter_cstr.as_ptr();
    let layer=WinDivertLayer::Network;
    let flags=WinDivertFlags::new().set_sniff();
    let w=unsafe {WinDivertOpen(filter, layer, 0i16, flags)};
    // 初始化 `WINDIVERT_ADDRESS`
    let mut address = <WINDIVERT_ADDRESS as std::default::Default>::default(); 
    let mut packet_buffer=vec![0u8; 65535];
    let mut packet_len=0u32;

    // 设置 Ctrl+C 退出处理
    // ctrlc::set_handler({
    //     let handle = w;
    //     move || {
    //         println!("Ctrl+C detected, cleaning up...");
    //         unsafe {
    //             WinDivertClose(handle);
    //         }
    //         println!("WinDivert handle closed. Exiting.");
    //         std::process::exit(0);
    //     }
    // })
    // .map_err(|_| "Failed to set Ctrl+C handler")?;
    while *stop_flag.lock().unwrap() {
        unsafe {
            let result=WinDivertRecv(
                w,
                packet_buffer.as_mut_ptr() as *mut _,
                packet_buffer.len() as u32,
                &mut packet_len,
                &mut address,
            );
            if result==false {
                sleep(Duration::from_millis(1000));
                //eprintln!("Failed to receive packet.");
                continue;
            }

        }
        let packet_data=&packet_buffer[..packet_len as usize];
        let ipv6_packet=match Ipv6Packet::new(packet_data) {
            Some(thepacket)=> thepacket,
            None=> {
                unsafe {
                    let _=WinDivertSend(
                        w,
                        packet_buffer.as_mut_ptr() as *mut _,
                        packet_buffer.len() as u32,
                        &mut packet_len,
                        &mut address,
                    );
                }
                continue;
            }
        };
        let icmpv6_packet=match Icmpv6Packet::new(ipv6_packet.payload()) {
            Some(thepacket)=> thepacket,
            None=> {
                unsafe {
                    let _=WinDivertSend(
                        w,
                        packet_buffer.as_mut_ptr() as *mut _,
                        packet_buffer.len() as u32,
                        &mut packet_len,
                        &mut address,
                    );
                }
                continue;
            },
        };
        if icmpv6_packet.get_icmpv6_type() != RouterAdvert {
            //debug!("It's not a RouterAdvert packet!");
            unsafe {
                let _=WinDivertSend(
                    w,
                    packet_buffer.as_mut_ptr() as *mut _,
                    packet_buffer.len() as u32,
                    &mut packet_len,
                    &mut address,
                );
            }
            continue;
        }
        let ra_packet = match RouterAdvertPacket::new(icmpv6_packet.packet()) {
            Some(packet) => packet,
            None => {
                unsafe {
                    let _=WinDivertSend(
                        w,
                        packet_buffer.as_mut_ptr() as *mut _,
                        packet_buffer.len() as u32,
                        &mut packet_len,
                        &mut address,
                    );
                }
                continue;
            },
        };
        for op in ra_packet.get_options() {
            if op.option_type != PrefixInformation {
                unsafe {
                    let _=WinDivertSend(
                        w,
                        packet_buffer.as_mut_ptr() as *mut _,
                        packet_buffer.len() as u32,
                        &mut packet_len,
                        &mut address,
                    );
                }
                continue;
            }
            let option_raw = op.to_bytes();
            let pfi = match PrefixInformationPacket::new(&option_raw) {
                Some(packet) => {
                    //debug!("Find PrefixInformationPacket ndp option!");
                    packet
                }
                None =>{
                    unsafe {
                        let _=WinDivertSend(
                            w,
                            packet_buffer.as_mut_ptr() as *mut _,
                            packet_buffer.len() as u32,
                            &mut packet_len,
                            &mut address,
                        );
                    }
                    continue;
                }
            };
            //let pkt_prefix_str = ipv6_addr_u8_to_string(pfi.payload());
            //debug!("IPv6 Prefix in packet is {}", pkt_prefix_str);
           
            let blacklist_mode = match BLACKLIST_MODE.lock() {//读取全局变量-黑名单模式
                Ok(guard)=> *guard,
                Err(e)=>{
                    eprint!("Failed to acquire lock for BLACKLIST_MODE: {}", e);
                    false
                },
            };
            let ipv6_prefix:Vec<Ipv6Net> = get_container_data();
            let is_prefix_in_list = ipv6_prefix.iter().any(|&prefix| prefix.addr().octets() == pfi.payload());
            let verdict = decide_verdict(blacklist_mode,is_prefix_in_list);
            if verdict {
                unsafe {
                    let _=WinDivertSend(
                        w,
                        packet_buffer.as_mut_ptr() as *mut _,
                        packet_buffer.len() as u32,
                        &mut packet_len,
                        &mut address,
                    );
                }
                continue;
            }
            //log_and_return(verdict, &pkt_prefix_str);
            //return verdict;
        }
    }
    unsafe {
        WinDivertClose(w);
    }
    println!("WinDivert handle closed. Exiting.")
}
#[cfg(windows)]
fn decide_verdict(blacklist_mode: bool, is_prefix_in_list: bool) -> bool {
    match (blacklist_mode, is_prefix_in_list) {
        (false, true) => true,//accept
        (true, false) => true,//accept
        _ => false,
    }
}
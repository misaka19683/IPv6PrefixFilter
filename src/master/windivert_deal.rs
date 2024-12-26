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
use pnet::packet::{ Packet,ipv6::Ipv6Packet,
    icmpv6::{Icmpv6Types::RouterAdvert,Icmpv6Packet,
    ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket}}};
use crate::globals::{get_container_data, BLACKLIST_MODE};
use ipnet::Ipv6Net;
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use std::{sync::{Arc, atomic::{AtomicBool,Ordering}},thread::sleep,time::Duration};
use crate::utils::ipv6_addr_u8_to_string;
use log::{info,debug};
macro_rules! send_packet {
    ($w:expr, $packet_buffer:expr, $packet_len:expr, $address:expr) => {
        unsafe {
            let sendsuccession=WinDivertSend(
                $w,
                $packet_buffer.as_mut_ptr() as *mut _,
                $packet_buffer.len() as u32,
                &mut $packet_len,
                &mut $address,
            );
            if sendsuccession==false {
                log::error!("Failed to send packet.");
                WinDivertSend(
                    $w,
                    $packet_buffer.as_mut_ptr() as *mut _,
                    $packet_buffer.len() as u32,
                    &mut $packet_len,
                    &mut $address,
                );
            }else{
                log::info!("Packet sent.");
            }
        }
    };
}
#[cfg(windows)]
pub fn the_process(stop_flag:Arc<AtomicBool>)  {

    use windivert_sys::WinDivertHelperCompileFilter;
    let w={
        let filter_cstr=CString::new("inbound and !loopback and icmpv6.Type==134").expect("CString::new failed");
        let filter=filter_cstr.as_ptr();
        let layer=WinDivertLayer::Network;

        //程序会检查 WinDivert 驱动程序是否已经安装在系统上。
        //如果 WinDivert 驱动程序没有安装，WinDivertOpen 函数将会失败，并返回一个特定的错误代码 ERROR_SERVICE_DOES_NOT_EXIST（值为 1060）。
        //这个错误代码是一个标准的 Windows 错误代码，表示请求的服务不存在。
        let flags=WinDivertFlags::new().set_no_installs();
        //let w=unsafe {WinDivertOpen(filter, layer, 0i16, flags)};
        let filter_object:*mut i8=0 as *mut i8;
        let objLen:u32=65535;
        let errStr:*mut *const i8=0 as *mut *const i8;
        let errPos:*mut u32=0 as *mut u32;
        let w=
            if unsafe{WinDivertHelperCompileFilter(filter,layer,filter_object,objLen,errStr,errPos)}==true {
                log::info!("Filter compiled successfully.");
                unsafe {WinDivertOpen(filter, layer, 0i16, flags)}
            }else{
                //println!("{}",errStr.to_str().unwrap());
                log::error!("Failed to compile filter.");
                return;
            };
        
        debug!("WinDivert handle opened.");
        w
    };
//=========================================================================================================================

    // 初始化 `WINDIVERT_ADDRESS`
    let mut address = <WINDIVERT_ADDRESS as std::default::Default>::default(); 
    let mut packet_buffer=vec![0u8; 65535];
    let mut packet_len=0u32;

    while stop_flag.load(Ordering::SeqCst) {
        
            //println!("Waiting for packets...");
            let result=unsafe { WinDivertRecv(
                w,
                packet_buffer.as_mut_ptr() as *mut _,
                packet_buffer.len() as u32,
                &mut packet_len,
                &mut address,
            )};
            if result==false {
                sleep(Duration::from_millis(100));
                debug!("Failed to receive packet.");
                continue;
            }
        
        // debug!("Received a packet.");
        let packet_data=&packet_buffer[..packet_len as usize];
        let ipv6_packet=match Ipv6Packet::new(packet_data) {
            Some(thepacket)=> thepacket,
            None=> {
                send_packet!(w, packet_buffer, packet_len, address);
                continue;
            }
        };
        debug!("It's an IPv6 packet.");
        let icmpv6_packet=match Icmpv6Packet::new(ipv6_packet.payload()) {
            Some(thepacket)=> thepacket,
            None=> {
                send_packet!(w, packet_buffer, packet_len, address);
                continue;
            },
        };
        debug!("It's an ICMPv6 packet.");
        if icmpv6_packet.get_icmpv6_type() != RouterAdvert {
            //debug!("It's not a RouterAdvert packet!");
            send_packet!(w, packet_buffer, packet_len, address);
            continue;
        }
        debug!("It's a RouterAdvert packet.");
        let ra_packet = match RouterAdvertPacket::new(icmpv6_packet.packet()) {
            Some(packet) => packet,
            None => {
                send_packet!(w, packet_buffer, packet_len, address);
                continue;
            },
        };
        debug!("It's a RouterAdvert packet with options.");
        for op in ra_packet.get_options() {
            if op.option_type != PrefixInformation {
                send_packet!(w, packet_buffer, packet_len, address);
                continue;
            }
            let option_raw = op.to_bytes();
            let pfi = match PrefixInformationPacket::new(&option_raw) {
                Some(packet) => {
                    //debug!("Find PrefixInformationPacket ndp option!");
                    packet
                }
                None =>{
                    send_packet!(w, packet_buffer, packet_len, address);
                    continue;
                }
            };
            let pkt_prefix_str = ipv6_addr_u8_to_string(pfi.payload());
            info!("IPv6 Prefix in packet is {}", pkt_prefix_str);
           
            let blacklist_mode = BLACKLIST_MODE.load(Ordering::SeqCst); //读取全局变量-黑名单模式

            let ipv6_prefix:Vec<Ipv6Net> = get_container_data();
            let is_prefix_in_list = ipv6_prefix.iter().any(|&prefix| prefix.addr().octets() == pfi.payload());
            if decide_verdict(blacklist_mode,is_prefix_in_list) {
                send_packet!(w, packet_buffer, packet_len, address);
                info!("Packet is allowed.");
                continue;
            }else {
                info!("Packet is not allowed.");
            }
            //log_and_return(verdict, &pkt_prefix_str);
            //return verdict;
        }
    }
    unsafe {WinDivertClose(w);}
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


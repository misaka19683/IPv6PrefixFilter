use windivert::{
    WinDivert,
    layer::NetworkLayer,
    packet::WinDivertPacket,
    prelude::WinDivertFlags};
use pnet::packet::{ Packet,ipv6::Ipv6Packet,
    icmpv6::{Icmpv6Types::RouterAdvert,Icmpv6Packet,
    ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket}}};
use log::{info,debug,error};
use std::{ sync::{ atomic::Ordering,Arc}};
use crate::{master::wdvt, utils::ipv6_addr_u8_to_string};
use crate::prefix_info::{PrefixInformationPacket, ToBytes};
use crate::globals::{get_container_data, BLACKLIST_MODE};

pub fn wdvt_process(){
    use tokio::runtime::Runtime;
    use tokio::task;
    let filter="inbound and !loopback and icmpv6.Type==134";
    let flags=WinDivertFlags::new();
    let mut w=
        WinDivert::network(filter, 0, flags).unwrap();
    let wdvt=Arc::new(w);
    let wdvt_clone=Arc::clone(&wdvt);
    let wdvt_clone2=Arc::clone(&wdvt);
    let mut buffer:Vec<u8>=Vec::new();
    let rt=Runtime::new().unwrap();

    let handle=task::spawn_blocking(move ||{

        loop {
            match wdvt_clone.recv(Some(&mut buffer.as_mut_slice())) {
                Ok(datapacket) => {
                    //let data=packet.data.into_owned();
                    //datapacket;
                    handle_packet(&datapacket, &wdvt_clone2);
                },
                Err(e) => {
                    match e{
                        _=>{error!("Error: {}", e);break;},
                    }
                },
            }
            //w.close(windivert::CloseAction::Nothing).unwrap();
        }
    });
    rt.block_on(async{
        let _=tokio::signal::ctrl_c().await;
        println!("Received SIGINT, shutting down...");
        handle.abort();
        //*wdvt.close(windivert::CloseAction::Nothing).unwrap();
        println!("Closed WinDivert handle.");
        handle.await.unwrap();
        info!("WinDivert handle closed.");
    });

}
fn handle_packet(packet:&WinDivertPacket<NetworkLayer>,wdt:&WinDivert<NetworkLayer>){
    let ipv6_packet=match Ipv6Packet::new(&packet.data){
        Some(v6_packet)=>v6_packet,
        None=>{
            send_twice(&wdt, &packet);
            return;
        },
    };
    let icmpv6_packet=match Icmpv6Packet::new(ipv6_packet.payload()) {
        Some(thepacket)=> thepacket,
        None=> {
            send_twice(&wdt, &packet);
            return;
        },
    };
    debug!("It's an ICMPv6 packet.");
    if icmpv6_packet.get_icmpv6_type() != RouterAdvert {
        //debug!("It's not a RouterAdvert packet!");
        send_twice(&wdt, &packet);
        return;
    }
    debug!("It's a RouterAdvert packet.");
    let ra_packet = match RouterAdvertPacket::new(icmpv6_packet.packet()) {
        Some(packet) => packet,
        None => {
            send_twice(&wdt, &packet);
            return;
        },
    };
    debug!("It's a RouterAdvert packet with options.");
    for op in ra_packet.get_options() {
        if op.option_type != PrefixInformation {
            send_twice(&wdt, &packet);
            return;
        }
        let option_raw = op.to_bytes();
        let pfi = match PrefixInformationPacket::new(&option_raw) {
            Some(packet) => {
                //debug!("Find PrefixInformationPacket ndp option!");
                packet
            },
            None =>{
                send_twice(&wdt, &packet);
                return;
            },
        };
        let pkt_prefix_str = ipv6_addr_u8_to_string(pfi.payload());
        info!("IPv6 Prefix in packet is {}", pkt_prefix_str);
        let blacklist_mode = BLACKLIST_MODE.load(Ordering::SeqCst);

        let ipv6_prefix = get_container_data();
        let is_prefix_in_list = ipv6_prefix.iter().any(|&prefix| prefix.addr().octets() == pfi.payload());
        match (blacklist_mode, is_prefix_in_list) {
            (false, true) => send_twice(&wdt, &packet),//accept
            (true, false) => send_twice(&wdt, &packet),//accept
            _ => {},//drop
        }
    }
}
fn send_twice(wdt:&WinDivert<NetworkLayer>,packet:&WinDivertPacket<NetworkLayer>){
    match wdt.send(&packet) {
        Ok(_) => debug!("Send once successfully!"),
        Err(_) => {
            match wdt.send(packet) {
                Ok(_) => debug!("Send twice successfully!"),
                Err(e) => error!("Send twice failed!"),
            }
        }
    }
}
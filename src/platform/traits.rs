use std::fmt::Debug;
use ipnet::Ipv6Net;
use log::info;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::icmpv6::Icmpv6Packet;
use pnet::packet::icmpv6::ndp::NdpOptionTypes::PrefixInformation;
use pnet::packet::icmpv6::ndp::RouterAdvertPacket;
use pnet::packet::ipv6::Ipv6Packet;
use pnet_macros_support::packet::Packet;
use crate::error::FilterError;
pub type FilterMode = bool;

pub trait FilterConfig :Debug {
    fn init(&mut self) -> Result<(),FilterError>;
    fn cleanup(&self) -> Result<(),FilterError>;
}

pub trait PacketProcessor :Debug {

    fn filter(&self) -> &Vec<Ipv6Net>;
    fn filter_mode(&self) -> FilterMode;

    fn capture_packet(&mut self) -> Result<(),FilterError>{Ok(())}
    
    fn analyze_packet(&mut self,data:&[u8]) -> Result<bool,FilterError>{
        use crate::platform::types::ToBytes;
        let ipv6_packet=if let Some(ipv6_packet)=Ipv6Packet::new(data) {ipv6_packet} else { return Ok(true) };
        let icmpv6_packet=if let Some(icmpv6_packet)=Icmpv6Packet::new(ipv6_packet.payload()){icmpv6_packet} else { return Ok(true) };
        let ra_packet=if let Some(ra_packet)=RouterAdvertPacket::new(icmpv6_packet.packet()) {ra_packet} else {return Ok(true)};

        for op in ra_packet.get_options() {
            if op.option_type !=PrefixInformation {continue;}

            let option_raw=op.to_bytes();
            let pfi=if let Some(pfi)=crate::platform::types::PrefixInformationPacket::new(&option_raw){pfi}else { continue; };

            if pfi.payload().len()!=16 { continue; }
            else {
                let array:[u8;16]=pfi.payload().try_into().unwrap();
                let ipv6addr=std::net::Ipv6Addr::from(array);
                info!("Recived an IPv6 Prefix: {}", ipv6addr);
            };

            let is_prefix_in_list=self.filter().iter().any(|prefix| {prefix.addr().octets()==pfi.payload()});
            let verdict=match (self.filter_mode(),is_prefix_in_list) {
                (false,false)=>{ Ok(true)},//黑名单模式，接受不在名单上的包
                (true,true)=>{ Ok(true)},//白名单模式，接受在名单上的包
                _ =>{Ok(false)},
            };
            return verdict;
        }
        Ok(true)
    }

    fn run(&mut self) -> Result<(),FilterError>;
}

pub struct PacketHandler<F,P> {
    filter_config:F,
    packet_process:P,
}

impl PacketHandler<Box<dyn FilterConfig>,Box<dyn PacketProcessor>> {
    pub fn new(filter_list:Vec<Ipv6Net>, filter_mode: FilterMode, interface_name:Option<String>) ->Self {
        let interface_name= match interface_name {
            None => None,
            Some(input_interface_name)=>{
                let interface_names_match=|iface:&NetworkInterface| iface.name == input_interface_name;
                let interfaces= datalink::interfaces();
                let interface=
                    interfaces.into_iter().find(interface_names_match)
                        .expect("Interface not found");
                Some(interface)
            },
        };
        
        #[cfg(target_os = "linux")]
        let (filter_config,packet_process)={
            let filter_config=crate::backend::LinuxFilter::new(0,interface_name);
            let packet_process=crate::backend::LinuxPacketProcessor { filter_mode, filter: filter_list };
            (Box::new(filter_config),Box::new(packet_process))
        };
        #[cfg(target_os = "windows")]
        todo!();
        
        Self {filter_config,packet_process}
    }

    pub fn run(&mut self) {
        self.filter_config.init().unwrap();
        self.packet_process.run().unwrap();
        self.filter_config.cleanup().expect("TODO: panic message");
    }
    pub fn clean(&mut self) {
        self.filter_config.cleanup().unwrap();
    }
}


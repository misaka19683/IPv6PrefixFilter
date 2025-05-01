use std::fmt::{Debug, Formatter};
use ipnet::Ipv6Net;
use pnet::datalink::{interfaces, NetworkInterface};

// use std::net::IpAddr;
// use ipnet::Ipv6Net;
use crate::error::FilterError;
pub type FilterMode = bool;

// #[derive(Debug,PartialEq,Eq)]
// pub enum FilterAction {
//     Accept,
//     Drop,
//     Pass,
// }


pub trait FilterConfig :Debug {
    fn init(&mut self) -> Result<(),FilterError>;
    fn cleanup(&self) -> Result<(),FilterError>;
}

pub trait PacketProcessor :Debug {
    fn capture_packet(&mut self) -> Result<(),FilterError>{Ok(())}
    
    fn analyze_packet(&mut self,data:&[u8]) -> Result<bool,FilterError>;

    fn run(&mut self) -> Result<(),FilterError>;
}

pub struct PacketHandler<F,P> {
    filter_config:F,
    packet_process:P,
}

impl  PacketHandler<Box<dyn FilterConfig>,Box<dyn PacketProcessor>> {
    pub fn new(filter_list:Vec<Ipv6Net>, filter_mode: FilterMode, interface_name:Option<String>) ->Self {
        let interface_name=if let Some(interface)=interface_name {
            let interface_names_match=|iface:&NetworkInterface| iface.name == interface;
            let interfaces= interfaces();
            let interface=
                interfaces.into_iter().find(interface_names_match)
                    .expect("Interface not found");
            Some(interface)
        }else {None};
        
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


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

    // fn capture_packet(&mut self) -> Result<(),FilterError>{Ok(())}
    
    fn analyze_packet(&mut self,data:&[u8]) -> Result<bool,FilterError>{
        use crate::platform::types::ToBytes;
        let Some(ipv6_packet)=Ipv6Packet::new(data) else { return Ok(true) };
        let Some(icmpv6_packet)=Icmpv6Packet::new(ipv6_packet.payload()) else { return Ok(true) };
        let Some(ra_packet)=RouterAdvertPacket::new(icmpv6_packet.packet()) else { return Ok(true) };

        for op in ra_packet.get_options() {
            if op.option_type !=PrefixInformation {continue}

            let option_raw=op.to_bytes();
            let Some(pfi)=super::types::PrefixInformationPacket::new(&option_raw) else { continue };

            if pfi.payload().len()!=16 { continue }
            else {
                let array:[u8;16]=pfi.payload().try_into().unwrap();//他会出什么错我不知道，不过提醒用户清理一下，仅对于linux用户
                let ipv6addr=std::net::Ipv6Addr::from(array);
                info!("Received an IPv6 Prefix: {}", ipv6addr);
            };

            let is_prefix_in_list=self.filter().iter().any(|prefix| {prefix.addr().octets()==pfi.payload()});
            let verdict=match (self.filter_mode(),is_prefix_in_list) {
                (false,false) =>{ Ok(true) },//黑名单模式，接受不在名单上的包
                (true, true)  =>{ Ok(true) },//白名单模式，接受在名单上的包
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
        self.filter_config.init().unwrap();//对于linux来说是在配置nft规则，如果失败，可能说明nft规则已经存在，应该提醒用户使用clear命令
        self.packet_process.run().unwrap();//我觉得这里不应该放错误处理，也许？毕竟这是一个抽象层，谁也不知道下面传来的错误是啥。
        self.filter_config.cleanup().expect("TODO: panic message");//对于windows，我在清理什么？
    }
    pub fn clean(&mut self) {
        self.filter_config.cleanup().unwrap();
    }
}


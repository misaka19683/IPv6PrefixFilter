use std::fmt::{Debug, Formatter};
use ipnet::Ipv6Net;
use pnet::datalink::NetworkInterface;
use crate::error::FilterError;
use crate::platform::traits::{FilterConfig, FilterMode, PacketProcessor};

pub struct WindowsFilter {
    interface_name: Option<NetworkInterface>,
}

impl Debug for WindowsFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl FilterConfig for WindowsFilter {
    fn init(&mut self) -> Result<(), FilterError> {
        todo!()
    }

    fn cleanup(&self) -> Result<(), FilterError> {
        todo!()
    }
}

pub struct WindowsPacketProcessor {
    filter_mode:FilterMode,
    filter:Vec<Ipv6Net>,
}

impl Debug for WindowsPacketProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PacketProcessor for WindowsPacketProcessor {
    // fn analyze_packet(&mut self, data: &[u8]) -> Result<bool, FilterError> {
    //     todo!()
    // }

    fn run(&mut self) -> Result<(), FilterError> {
        todo!()
    }
}
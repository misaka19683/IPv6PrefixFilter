use std::fmt::{Debug, Formatter};
use std::net::IpAddr;
use crate::error::FilterError;
pub type FilterMode = bool;

#[derive(Debug,PartialEq,Eq)]
pub enum FilterAction {
    Accept,
    Drop,
    Pass,
}

// pub trait PacketProcessor :Debug + PartialEq {
//     
//     fn init(&mut self) -> Result<(),FilterError>;
//     
//     fn cleanup(&self) -> Result<(),FilterError>;
//     
//     fn capture_packet(&mut self) -> Result<Option<IpAddr>,FilterError>;
//     
//     // fn analyze_packet(&self,packet: )
// }

pub trait FilterConfig :Debug {
    fn init(&mut self) -> Result<(),FilterError>;
    fn cleanup(&self) -> Result<(),FilterError>;
}

pub trait PacketProcessor :Debug + PartialEq {
    fn capture_packet(&mut self) -> Result<(),FilterError>;
    
    fn analyze_packet(&mut self) -> Result<(),FilterError>;
}


#![allow(non_snake_case)]

pub mod master;

use ipnet::Ipv6Net;
use pnet::datalink::NetworkInterface;

#[derive(Default)]
pub struct AppState {
    pub queue_num: u16,
    pub blacklist_mode: bool,
    pub prefixes: Vec<Ipv6Net>,
    pub interface: Option<NetworkInterface>,
}

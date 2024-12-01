use clap::Parser;
use std::net::Ipv6Addr;
use crate::globals::add_to_container;

#[derive(Parser, Debug)]
#[command(version,about,long_about=None)]

struct Args {
    #[arg(short='p', long, default_value = "")]
    ipv6_prefix: Ipv6Addr,

}
pub fn get_prefix() -> [u8; 16] {
    let args=Args::parse();
    let ipv6_prefix = args.ipv6_prefix.octets();
    add_to_container(ipv6_prefix);
    return ipv6_prefix;
}
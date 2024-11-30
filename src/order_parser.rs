use clap::Parser;
use std::net::Ipv6Addr;

#[derive(Parser, Debug)]
#[command(version,about,long_about=None)]

struct Args {
    #[arg(short='p', long, default_value = "")]
    ipv6_prefix: Ipv6Addr,

}
pub fn get_prefix() -> [u8; 16] {
    let args=Args::parse();
    let ipv6_prefix = args.ipv6_prefix.octets();
    return ipv6_prefix;
}
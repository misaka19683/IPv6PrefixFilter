use clap::Parser;
use std::net::Ipv6Addr;
use crate::globals::add_to_container;

#[derive(Parser, Debug)]
#[command(version,about,long_about=None)]

struct Args {
    #[arg(short='p', long, default_value = "")]
    ipv6_prefix: Ipv6Addr,
    
}
pub fn push_prefix()  {
    let args=Args::parse();
    //let ipv6_prefix = args.ipv6_prefix.octets();
    add_to_container(args.ipv6_prefix);
}
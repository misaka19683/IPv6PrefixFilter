use clap::Parser;
use crate::cli::{Cli,Commands};
use ipnet::Ipv6Net;
use log::info;
use crate::platform::traits::PacketHandler;
mod cli;
mod error;
mod platform;
mod backend;

fn main() {
    println!("Hello, world!");
    let cli=Cli::parse();
    match cli.command { 
        Commands::Run {
            interface_name,
            filter_mode,
            filter
        } => {
            let mut packet_handler = PacketHandler::new(filter,filter_mode,interface_name);
            info!("Starting packet processing...");
            packet_handler.run();
        }
        Commands::Clear =>{
            let net: Ipv6Net = "::/128".parse().unwrap();
            let filter=vec![net];
            let filter_mode=false;
            let interface_name=None;
            let mut packet_handler = PacketHandler::new(filter,filter_mode,interface_name);
            packet_handler.clean();
            println!("clean up nft rules");
            // 非正常clear，需要先创建对象，然后再清除相关代码，对于windows也许并不适用，但是一定要有对象才能调用对象的方法啊！
        }
    }
}

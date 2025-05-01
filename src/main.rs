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
        Commands::Clear =>{}
    }
}

use clap::{Parser,Subcommand};
use ipnet::Ipv6Net;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the filter or packet processor
    Run {
        // #[arg(short, long)]
        // queue_num: Option<u32>,

        #[arg(short='i', long)]
        interface_name: Option<String>,

        #[arg(short='b', long)]
        filter_mode: bool,

        #[arg(short='p', long, value_delimiter = ',')]
        filter: Vec<Ipv6Net>,
    },
    Clear,
    // /// Capture packets
    // Capture {},
    // 
    // /// Analyze a packet
    // Analyze {
    //     #[arg(short, long)]
    //     data: String, // Base64 encoded packet data for simplicity
    // },



}

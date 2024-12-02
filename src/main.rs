use clap::{Parser, Subcommand};
use std::net::Ipv6Addr;
// 引用自己的代码
use IPv6PrefixFilter::globals::add_to_container;
use IPv6PrefixFilter::handle::handle_run;
//use IPv6PrefixFilter::queue::{process_queue, start_queue};

/// 程序的命令行参数结构体
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short = 'p', long, value_parser = clap::value_parser!(Ipv6Addr))]
    ipv6_prefixes: Vec<Ipv6Addr>,

    #[arg(short = 'i', long)]
    interface: Option<String>,

    #[arg(short = 'b', long)]
    blacklist_mode: bool,

    #[arg(short = 'v', long)]
    verbose: bool,

    #[arg(short = 'd', long)]
    daemon: bool,

    #[arg(long = "disable-nft-autoset")]
    disable_nft_autoset: bool,
}

/// 定义程序支持的命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    Run {
        #[arg(short = 'p', long, value_parser = clap::value_parser!(Ipv6Addr))]
        ipv6_prefix: Option<Ipv6Addr>,
    },
    Clear,
    Daemon,
    Version,
}

/// 清理操作
pub fn handle_clear(args: &Args) {
    println!("Clearing nftables rules.");
    if args.verbose {
        println!("Verbose mode enabled.");
    }
}

fn main() {
    // 解析命令行参数
    let args = Args::parse();
    add_to_container(args.ipv6_prefixes[0]);

    // 根据命令执行不同操作
    match args.command {
        Some(Commands::Run { .. }) => {
            handle_run(); // 传递参数给`handle_run`
        }
        Some(Commands::Clear) => {
            handle_clear(&args); // 传递参数给`handle_clear`
        }
        Some(Commands::Daemon) => {
            println!("Running as daemon.");
        }
        Some(Commands::Version) => {
            println!("Version 1.0.0");
        }
        None => {
            println!("No command provided. Use --help for help.");
        }
    }

    // 输出所有的IPv6前缀
    for prefix in &args.ipv6_prefixes {
        println!("Allowed IPv6 prefix: {}", prefix);
    }
}

use clap::{Parser, Subcommand};
use tokio;
use env_logger;
use env_logger::{Builder, Target};
use ipnet::Ipv6Net;
use log::{debug, info, warn};
use std::sync::atomic::Ordering;
// 引用自己的代码
//#[cfg(target_os = "linux")]
// use IPv6PrefixFilter::daemon;

use IPv6PrefixFilter::{ master::*,globals::*};

/// Use IPv6PrefixFilter [COMMAND] --help to see the detail help for each subcommand.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {   
    #[command(subcommand)]
    command: Option<Commands>,

    // /// Specify the allowed IPv6 prefixes. Multiple prefixes can be allowed by repeating the `-p` option.
    // #[arg(short = 'p', long, value_parser = clap::value_parser!(Ipv6Net))]
    // ipv6_prefixes: Vec<Ipv6Net>,

    /// Display detailed runtime information. The default log level is warning. Use -v to set to info, and -vv for debug.
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8
}

/// 定义程序支持的命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the program (in the foreground).
    Run {
        /// Specify the allowed IPv6 prefixes. Multiple prefixes can be allowed by repeating the `-p` option.
        #[arg(short = 'p', long, value_parser = clap::value_parser!(Ipv6Net))]
        ipv6_prefixs: Vec<Ipv6Net>,
        /// Specify the wan interface.
        #[arg(short = 'i', long)]
        interface: Option<String>,
        /// Enable blacklist mode. Prefixes specified with `-p` will be blocked.
        #[arg(short = 'b', long)]
        blacklist_mode: bool,
        /// Disable the feature of auto set nftables rules.
        #[arg(long = "disable-nft-autoset")]
        disable_nft_autoset: bool
    },
    // 清理nftables规则
    /// Clear the nft rules set by the program, especially when the program exits improperly without executing the cleanup process.
    Clear,
    /// Run as a daemon process.
    // Daemon, //TODO: Waiting for implementation.
    /// Print version info.
    Version
}
#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args = Args::parse();

    // 初始化日志记录
    match args.verbose {
        0 => env_logger::init(),
        1 => {
            Builder::new().filter_level(log::LevelFilter::Info).target(Target::Stdout).init();
            info!("Logging level has been set to info.")
        },
        _ => {
            Builder::new().filter_level(log::LevelFilter::Debug).target(Target::Stdout).init();
            debug!("Logging level has been set to info.")
        }
    };
    // 根据命令执行不同操作
    match args.command {
        Some(Commands::Run { ipv6_prefixs, interface, blacklist_mode, disable_nft_autoset}) => {
            debug!("Running with prefix: ");

            for prefix in ipv6_prefixs.iter() {
                add_to_container(*prefix);
            }

            if let Some(interface)= interface{
                set_interface_name(interface);
            };
            if blacklist_mode{BLACKLIST_MODE.store(true, Ordering::SeqCst);}

            #[cfg(target_os = "linux")]
            handle_run(disable_nft_autoset);
            #[cfg(windows)]
            if disable_nft_autoset{
                warn!("The disable_nft_autoset option is not supported on Windows.");
            }else{handle_run().await;}
        }
        Some(Commands::Clear) => {
            #[cfg(target_os = "linux")]
            warn!("Only supported on Linux.");
            #[cfg(target_os = "linux")]
            handle_clear(); // 传递参数给`handle_clear`
        }
        // TODO: Waiting for implementation
        // Some(Commands::Daemon) => {
        //     #[cfg(target_os = "linux")]
        //     daemon::daemon_run().expect("Failed to start daemon."); // 启动守护进程
        //     println!("Running as daemon.");
        // }
        Some(Commands::Version) => {
            println!("Version 1.0.0");
        }
        _ => {
            println!("No command provided. Use --help for help.");
        }
    }

    // 输出所有的IPv6前缀
    let prefixs=get_container_data();
    for prefix in prefixs.iter() {
        println!("Allowed IPv6 prefix: {}", prefix);
    }
}

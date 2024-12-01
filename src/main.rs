//use IPv6PrefixFilter::globals::QUEUE_NUM;
use IPv6PrefixFilter::nft;
use IPv6PrefixFilter::queue::{start_queue, process_queue};
use IPv6PrefixFilter::error::handle_error;

//mod nft_old;

//use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
//use std::thread;
//use std::time::Duration;
use log::info;//, error, warn}; // 导入 log 宏
use env_logger;
// use queue::{start_queue, process_queue,end_queue};
use clap::{Parser, Subcommand};
use std::net::Ipv6Addr;

// fn main() {
//     // 初始化日志记录
//     env_logger::init();
//     let queue_num = 0; // 队列号初始化为 0
//     // 设置退出信号捕获
//     let running = Arc::new(AtomicBool::new(true));
//     let r = running.clone();
//     //let r2 = running.clone(); // 克隆 running 用于队列线程
 

//     // 初始化 nftables
//     info!("Setting up nftables...");
//     nft::setup_nftables().expect("Failed to set up nftables");

//     // 启动队列监听器
//     info!("Starting NFQUEUE...");
//     let mut queue=start_queue(queue_num ).expect("Failed to start NFQUEUE");

//  // 捕获 Ctrl+C 信号（可以通过外部库如 ctrlc 处理）
//  ctrlc::set_handler(move || {
//     println!("Received Ctrl+C, stopping...");
//     r.store(false, Ordering::SeqCst);  // 停止队列处理
// })
// .expect("Error setting Ctrl+C handler");

// // 主循环
// while running.load(Ordering::SeqCst) {
//     process_queue(&mut queue).expect("Failed to process NFQUEUE");  // 处理队列中的数据包
// }
//     end_queue(&mut queue,queue_num ).expect("Failed to end NFQUEUE");  // 停止队列监听器
//     drop(queue); // 释放队列资源
//     // 清理 nftables 规则
//     info!("Cleaning up nftables...");
//     nft::delete_nftables().expect("Failed to delete nftables");


//     info!("Program exited cleanly.");
// }


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

    #[arg(short = 'h', long)]
    help: bool,
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

/// 处理`run`命令
pub fn handle_run(args: &Args) {
    // 初始化日志记录
    env_logger::init();
    //let queue_num = 0; // 队列号初始化为 0
    // 使用 Arc 和 Mutex 来共享 stop_flag

    let stop_flag = Arc::new(Mutex::new(false));
 

    // 初始化 nftables
    info!("Setting up nftables...");
    nft::setup_nftables().expect("Failed to set up nftables");

    // 启动队列监听器
    info!("Starting NFQUEUE...");
    let mut queue=start_queue().expect("Failed to start NFQUEUE");

 // 捕获 Ctrl+C 信号并设置 stop_flag 为 true
 {
    let stop_flag = Arc::clone(&stop_flag);
    ctrlc::set_handler(move || {
        println!("Caught Ctrl+C, throwing interrupted error...");
        let mut stop_flag = stop_flag.lock().unwrap();
        *stop_flag = true; // 设置 stop_flag，允许处理程序退出
    }).expect("Error setting Ctrl+C handler");
}

  // 处理队列中的数据包
    match process_queue(&mut queue, stop_flag) {
        Ok(_) => info!("Queue processing completed successfully."),
        Err(e) => handle_error(e),
    }

    // // 清理 nftables 规则
    // info!("Cleaning up nftables...");
    // nft::delete_nftables().expect("Failed to delete nftables");


    // info!("Program exited cleanly.");
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

    // 根据命令执行不同操作
    match args.command {
        Some(Commands::Run { .. }) => {
            handle_run(&args); // 传递参数给`handle_run`
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

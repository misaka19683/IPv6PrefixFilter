//use clap::{Parser, Subcommand};
use std::env::Args;
use log::info;
use std::sync::Arc;
use crate::{nft, order_parser::push_prefix};
use crate::queue::{start_queue, process_queue};
use crate::error::handle_error;
use std::sync::Mutex;
use env_logger;
/// 处理`run`命令
pub fn handle_run(args: &Args) {
    // 初始化日志记录
    env_logger::init();
    // 设置退出信号捕获
    // let running = Arc::new(AtomicBool::new(true));
    let stop_flag = Arc::new(Mutex::new(false));

    nft::setup_nftables().expect("Failed to set up nftables");

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


    push_prefix();

    
    match process_queue(&mut queue, stop_flag) {
            Ok(_) => info!("Queue processing completed successfully."),
            Err(e) => handle_error(e),
        }
}

/// 清理操作
pub fn handle_clear(args: &Args) {
    nft::delete_nftables().expect("Failed to clear nftables");
}

mod nft;
mod queue;
mod prefix_info;
mod order_parser;
mod utils;
//mod nft_old;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
//use std::thread;
//use std::time::Duration;
use log::info;//, error, warn}; // 导入 log 宏
use env_logger;
use queue::{start_queue, process_queue,end_queue}; // 导入 env_logger

fn main() {
    // 初始化日志记录
    env_logger::init();
    let queue_num = 0; // 队列号初始化为 0
    // 设置退出信号捕获
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    //let r2 = running.clone(); // 克隆 running 用于队列线程
 

    // 初始化 nftables
    info!("Setting up nftables...");
    nft::setup_nftables().expect("Failed to set up nftables");

    // 启动队列监听器
    info!("Starting NFQUEUE...");
    let mut queue=start_queue(queue_num ).expect("Failed to start NFQUEUE");

 // 捕获 Ctrl+C 信号（可以通过外部库如 ctrlc 处理）
 ctrlc::set_handler(move || {
    println!("Received Ctrl+C, stopping...");
    r.store(false, Ordering::SeqCst);  // 停止队列处理
})
.expect("Error setting Ctrl+C handler");

// 主循环
while running.load(Ordering::SeqCst) {
    process_queue(&mut queue).expect("Failed to process NFQUEUE");  // 处理队列中的数据包
}
    end_queue(&mut queue,queue_num ).expect("Failed to end NFQUEUE");  // 停止队列监听器
    drop(queue); // 释放队列资源
    // 清理 nftables 规则
    info!("Cleaning up nftables...");
    nft::delete_nftables().expect("Failed to delete nftables");


    info!("Program exited cleanly.");
}


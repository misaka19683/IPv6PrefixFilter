

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
 // 导入 env_logger



fn main() {
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


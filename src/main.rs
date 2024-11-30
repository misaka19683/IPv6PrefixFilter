mod nft;
mod queue;
//mod nft_old;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use log::{info, error, warn}; // 导入 log 宏
use env_logger; // 导入 env_logger

fn main() {
    // 初始化日志记录
    env_logger::init();

    // 设置退出信号捕获
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let r2 = running.clone(); // 克隆 running 用于队列线程
    // 捕获 Ctrl+C 信号以触发清理
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, exiting...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // 初始化 nftables
    info!("Setting up nftables...");
    nft::setup_nftables().expect("Failed to set up nftables");

    // 启动队列监听器
    info!("Starting NFQUEUE...");
    let queue_thread = thread::spawn(move || {
        if let Err(e) = queue::start_queue(r2.clone()) {
            error!("Error in queue: {}", e);
        }
    });

    // 主线程等待退出信号
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
    }

    // 清理 nftables 规则
    info!("Cleaning up nftables...");
    nft::delete_nftables().expect("Failed to delete nftables");

    // 等待队列线程结束
    if let Err(e) = queue_thread.join() {
        warn!("Error waiting for queue thread: {:?}", e);
    }

    info!("Program exited cleanly.");
}


mod nft;
mod queue;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // 设置退出信号捕获
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let r2 = running.clone(); // 克隆 running 用于队列线程
    // 捕获 Ctrl+C 信号以触发清理
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, exiting...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // 初始化 nftables
    println!("Setting up nftables...");
    nft::setup_nftables();

    // 启动队列监听器
    println!("Starting NFQUEUE...");
    let queue_thread = thread::spawn(move || {
        if let Err(e) = queue::start_queue(r2.clone()) {
            eprintln!("Error in queue: {}", e);
        }
    });

    // 主线程等待退出信号
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
    }

    // 清理 nftables 规则
    println!("Cleaning up nftables...");
    nft::cleanup_nftables();

    // 等待队列线程结束
    if let Err(e) = queue_thread.join() {
        eprintln!("Error waiting for queue thread: {:?}", e);
    }

    println!("Program exited cleanly.");
}

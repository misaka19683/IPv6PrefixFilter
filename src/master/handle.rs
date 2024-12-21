use log::{info,warn,debug};
use std::sync::{Arc, Mutex};

use crate::master::*;

#[cfg(windows)]
use windivert_deal::*;

#[cfg(target_os = "linux")]
use crate::error::handle_error;
#[cfg(target_os = "linux")]
use crate::globals::{clear_container,clear_interface_name};


//use crate::master::queue::{process_queue, start_queue};

#[cfg(target_os = "linux")]
pub fn handle_init(){
    start_queue().unwrap();
}
/// 处理`run`命令
#[cfg(target_os = "linux")]
pub fn handle_run(disable_nft_autoset:bool) {
    info!("IPv6PrefixFilter start running on Linux...");
    
    // 设置退出信号捕获
    // let running = Arc::new(AtomicBool::new(true));
    let stop_flag = Arc::new(Mutex::new(true));

    if disable_nft_autoset {
        warn!("nftables rule set is not enabled, please set nftables rules manually");
    }else {
        // 初始化 nftables
        info!("Setting up nftables...");
        setup_nftables().expect("Failed to set up nftables");
    }
    // 启动队列监听器
    info!("Starting NFQUEUE listen...");
    let mut queue = start_queue().expect("Failed to start NFQUEUE");

    // 捕获 Ctrl+C 信号并设置 stop_flag 为 true
    {
        let stop_flag = Arc::clone(&stop_flag);
        ctrlc::set_handler(move || {
            warn!("Caught Ctrl+C, throwing interrupted error...");
            let mut stop_flag = stop_flag.lock().unwrap();
            *stop_flag = false; // 设置 stop_flag，允许处理程序退出
        })
        .expect("Error setting Ctrl+C handler");
    }

    match process_queue(&mut queue, stop_flag) {
        Ok(_) => {},
        Err(e) => handle_error(e),
    }
}

#[cfg(windows)]
pub fn handle_run() {
    info!("IPv6PrefixFilter start running on Windows...");

    let stop_flag = Arc::new(Mutex::new(true));
    {
        let stop_flag = Arc::clone(&stop_flag);
        ctrlc::set_handler(move || {
            println!("Caught Ctrl+C, throwing interrupted error...");
            let mut stop_flag = stop_flag.lock().unwrap();
            *stop_flag = false; // 设置 stop_flag，允许处理程序退出
        })
        .expect("Error setting Ctrl+C handler");
    }
    info!("start_deal");
    debug!("debug_start_deal");
    the_process(stop_flag);
}

/// 清理操作
#[cfg(target_os = "linux")]
pub fn handle_clear() {
    delete_nftables().expect("Failed to clear nftables");
    // clear_interface_name();
    // clear_container();
}
#[cfg(target_os = "linux")]
pub fn handle_end(){
    delete_nftables().expect("Failed to clear nftables");
    clear_interface_name();
    clear_container();
}
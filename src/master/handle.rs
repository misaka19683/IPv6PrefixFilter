use log::{info,warn};
use crate::master::*;

#[cfg(windows)]
use windivert_deal::*;

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
    
    if disable_nft_autoset {
        warn!("nftables rule set is not enabled, please set nftables rules manually");
    }else {
        // 初始化 nftables
        info!("Setting up nftables...");
        setup_nftables().expect("Failed to set up nftables");
    }
    // 启动队列监听器
    info!("Starting NFQUEUE listen...");
    process_queue();
    delete_nftables().expect("Failed to clear nftables");
}
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::sync::Arc;
#[cfg(windows)]
use crate::master::windivert_deal::the_process;
#[cfg(windows)]
pub fn handle_run() {
    info!("IPv6PrefixFilter start running on Windows...");

    let stop_flag = Arc::new(AtomicBool::new(true));
    {
        let stop_flag = Arc::clone(&stop_flag);
        ctrlc::set_handler(move || {
            warn!("Caught Ctrl+C, throwing interrupted error...");
            stop_flag.store(false, Ordering::SeqCst); // 设置 stop_flag，允许处理程序退出
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
#[cfg(target_os = "linux")]
use log::{info,warn,debug};
#[cfg(target_os = "linux")]
use crate::master::*;


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
    delete_nftables()
    .expect("Failed to clear nftables, please use 'sudo nft delete table ip6 rafilter' to clear manually");
}

#[cfg(windows)]
pub async fn handle_run(){
    use log::info;
    use crate::master::wdvt::wdvt_process;
    info!("IPv6PrefixFilter start running on Windows...");
    wdvt_process().await;
    info!("IPv6PrefixFilter end running on Windows...");
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
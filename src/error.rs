use thiserror::Error;
use log::{error, info};
use crate::nft::delete_nftables;
//use crate::queue::{self, end_queue};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Ctrl+C signal received, shutting down.")]
    CtrlC,
    
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Failed to process queue: {0}")]
    QueueError(String),
    
    #[error("Unexpected error: {0}")]
    Unexpected(String),

    
}

// 一个统一的 Result 类型，方便其他模块使用
pub type Result<T> = std::result::Result<T, AppError>;

pub fn handle_error(err: AppError) {
    match err {
        AppError::CtrlC => {
            
            delete_nftables().unwrap();
            println!("Received Ctrl+C, exiting...");
            info!("Program exited cleanly.");
            //程序结束
        }
        AppError::IoError(e) => {
            delete_nftables().unwrap();
            eprintln!("I/O error: {}", e);
        }
        AppError::QueueError(msg) => {
            delete_nftables().unwrap();
            eprintln!("Queue error: {}", msg);
        }
        AppError::Unexpected(msg) => {
            delete_nftables().unwrap();
            eprintln!("Unexpected error: {}", msg);
        }
    }
}

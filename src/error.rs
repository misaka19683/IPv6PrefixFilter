#[cfg(target_os = "linux")]
use crate::master::{handle_clear, handle_init};
use log::{error, info};
use thiserror::Error;

//错误由两种类型,一种是不能处理的错误，一种是可以处理的错误。
//不能处理的错误必须panic,现在统计共有多少种错误可能出现
//首先
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Ctrl+C signal received, shutting down.")]
    Interrupt,

    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to start queue: {0}")]
    QueueStartError(String),

    #[error("Failed to process queue: {0}")]
    QueueProcessError(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

// 一个统一的 Result 类型，方便其他模块使用
pub type Result<T> = std::result::Result<T, AppError>;

pub fn handle_error(err: AppError) {
    match err {
        AppError::Interrupt => {
            #[cfg(target_os = "linux")]
            handle_clear();
            println!("Received Ctrl+C, exiting...");
            info!("Program exited cleanly.");
            //程序结束
        }
        AppError::IoError(e) => {
            #[cfg(target_os = "linux")]
            handle_clear();
            eprintln!("I/O error: {}", e);
        }
        AppError::QueueStartError(msg) => {
            eprintln!("Failed to start queue,try again. Error: {}", msg);
            #[cfg(target_os = "linux")]
        handle_init(); 
        }
        AppError::QueueProcessError(msg) => {
            #[cfg(target_os = "linux")]
            handle_clear();
            eprintln!("Queue error: {}", msg);
        }

        AppError::Unexpected(msg) => {
            #[cfg(target_os = "linux")]
            handle_clear();
            eprintln!("Unexpected error: {}", msg);
        } //AppError::_ =>{}
    }
}

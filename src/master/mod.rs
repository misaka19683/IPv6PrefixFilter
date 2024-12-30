#[cfg(target_os = "linux")]
mod nft;
#[cfg(target_os = "linux")]
mod queue;
#[cfg(target_os = "linux")]
mod handle;
#[cfg(target_os = "linux")]
use nft::{setup_nftables,delete_nftables};
#[cfg(target_os = "linux")]
use queue::{start_queue,process_queue};
#[cfg(target_os = "linux")]
pub use handle::*;

#[cfg(windows)]
mod wdvt;
#[cfg(windows)]
mod handle;
#[cfg(windows)]
pub use handle::*;
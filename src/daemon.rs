#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use daemonize::Daemonize;
#[cfg(target_os = "linux")]
use std::io;
#[cfg(target_os = "linux")]
use crate::master::{handle_clear, handle_run};
#[cfg(target_os = "linux")]
/// 启动守护进程并运行程序主体
pub fn daemon_run() -> io::Result<()> {
    let stdout = File::create("/tmp/IPV6PrefixFilter.out")?;
    let stderr = File::create("/tmp/IPV6PrefixFilter.err")?;
    // 创建 Daemonize 实例并配置
    let daemonize = Daemonize::new()
    .pid_file("/tmp/test.pid") // Every method except `new` and `start`
    .chown_pid_file(true)      // is optional, see `Daemonize` documentation
    .working_directory("/tmp") // for default behaviour.
    .user("nobody")
    .group("daemon") // Group name
    .group(2)        // or group id.
    .umask(0o777)    // Set umask, `0o027` by default.
    .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
    .stderr(stderr)  // Redirect stderr to `/tmp/daemon.err`.
    .privileged_action(|| "Executed before drop privileges");
    // // 启动守护进程
    match daemonize.start() {
        Ok(_) => {
            // 调用程序主体函数
            handle_run();
            // 如果 handle_run 返回，这里将被执行
            handle_clear();
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            
        }
    }
    Ok(())
}
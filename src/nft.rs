use std::process::Command;

/// 添加 nftables 表、链和规则
pub fn setup_nftables() {
    run_nft_command(&["add", "table", "ip6", "rafilter"]);
    run_nft_command(&[
        "add", "chain", "ip6", "rafilter", "input",
        "{", "type", "filter", "hook", "input", "priority", "0", ";", "}"
    ]);
    run_nft_command(&[
        "add", "rule", "ip6", "rafilter", "input", "icmpv6",
        "type", "134", "queue", "num", "0"
    ]);
    println!("nftables setup completed.");
}

/// 删除 nftables 表（自动删除链和规则）
pub fn cleanup_nftables() {
    run_nft_command(&["delete", "table", "ip6", "rafilter"]);
    println!("nftables cleanup completed.");
}

/// 执行 nft 命令的通用函数
fn run_nft_command(args: &[&str]) {
    let output = Command::new("nft")
        .args(args)
        .output()
        .expect("Failed to execute nft command");

    if output.status.success() {
        println!("nft command executed: {:?}", args);
    } else {
        eprintln!(
            "Error executing nft command {:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}


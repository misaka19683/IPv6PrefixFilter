#[cfg(target_os = "windows")]
fn config() {
    println!("cargo:rustc-link-arg=/MT"); // 强制使用静态运行时库
}

#[cfg(target_os = "linux")]
fn config() {}

fn main() {
    config();

}

fn main() {
    println!("cargo:rustc-link-arg=/MT"); // 强制使用静态运行时库
}

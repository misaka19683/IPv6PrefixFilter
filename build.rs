// #[cfg(target_os = "windows")]
// use std::{fs::copy,path::Path};
#[cfg(target_os = "windows")]
fn config() {
    // println!("cargo:rustc-link-arg=/MT"); // 强制使用静态运行时库
    println!("cargo:rustc-link-lib=static=Packet");
    println!("cargo:rustc-link-search=native=./lib/npcap-sdk-1.13/Lib/x64");

    // println!("cargo:rustc-link-search=native=./lib/windivert/x64");
    // let dll_path = Path::new("./lib/windivert/x64/WinDivert.dll");
    // let out_dir=std::env::var("OUT_DIR").expect("OUT_DIR not found");
    // let target_path = Path::new(&out_dir).join("WinDivert.dll");
    // if !target_path.exists() {
    //     copy(dll_path, target_path).unwrap();
    // }
    println!("cargo:rustc-link-search=native=./lib/windivert/x64");
    println!("cargo:rustc-link-lib=dylib=WinDivert");
}

#[cfg(target_os = "linux")]
fn config() {}

fn main() {
    config();

}

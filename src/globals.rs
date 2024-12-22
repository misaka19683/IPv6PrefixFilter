use once_cell::sync::Lazy;
use std::{collections::BTreeMap, sync::{atomic::AtomicBool, Mutex, RwLock}};
use ipnet::Ipv6Net;
use pnet::datalink::{interfaces,NetworkInterface,MacAddr};
//{interfaces, NetworkInterface};
// 全局变量定义
pub static QUEUE_NUM: u16 = 0;
pub static BLACKLIST_MODE: AtomicBool = AtomicBool::new(false);
pub static GLOBAL_CONTAINER: Lazy<Mutex<Vec<Ipv6Net>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static INTERFACE_NAME: Lazy<Mutex<Option<NetworkInterface>>> = Lazy::new(|| Mutex::new(None));
// pub static LAN_IPV6_ADDRESSES_LIST:Lazy<RwLock<Vec<Ipv6Net>>>=Lazy::new(|| RwLock::new(Vec::new()));
pub static LAN_IPV6_ADDRESSES_LIST:Lazy<RwLock<BTreeMap<MacAddr,Vec<Ipv6Net>>>>=Lazy::new(|| RwLock::new(BTreeMap::new()));
// GLOBAL_CONTAINER 方法
/// 向全局容器添加一个截断后的 IPv6 网络。
pub fn add_to_container(ip: Ipv6Net) {
    let mut container = GLOBAL_CONTAINER.lock().unwrap();
    container.push(ip.trunc());
}

/// 获取全局容器中的所有 IPv6 网络数据。
pub fn get_container_data() -> Vec<Ipv6Net> {
    let container = GLOBAL_CONTAINER.lock().unwrap();
    container.clone()
}

/// 清空全局容器中的所有数据。
pub fn clear_container() {
    let mut container = GLOBAL_CONTAINER.lock().unwrap();
    container.clear();
}

// INTERFACE_NAME 方法
/// 设置全局接口名称。
pub fn set_interface_name(input_interface_name: String) {
    let interface_names_match=|iface:&NetworkInterface| iface.name == input_interface_name;
    let interfaces= interfaces();
    let interface=
        interfaces.into_iter()
                .filter(interface_names_match)
                .next()
                .expect("Interface not found");
    let mut interface_name = INTERFACE_NAME.lock().unwrap();
    *interface_name = Some(interface);
}

/// 获取全局接口名称。
pub fn get_interface_name() -> Option<NetworkInterface> {
    let interface_name = INTERFACE_NAME.lock().unwrap();
    interface_name.clone()
}

/// 清除全局接口名称。
pub fn clear_interface_name() {
    let mut interface_name = INTERFACE_NAME.lock().unwrap();
    *interface_name = None;
}



// 向LAN_NEIBORHOOD_IPV6写入IPv6网络
pub fn set_lan_ipv6_address(mac: MacAddr, ipv6_network: Ipv6Net) {
    let mut write_guard = LAN_IPV6_ADDRESSES_LIST.write().unwrap();
    write_guard.entry(mac).or_insert_with(Vec::new).push(ipv6_network);
}

// 从LAN_NEIBORHOOD_IPV6读取IPv6网络列表
pub fn get_lan_ipv6_addresses(mac:&MacAddr) -> Option<Vec<Ipv6Net>> {
    let read_guard = LAN_IPV6_ADDRESSES_LIST.read().unwrap();
    //read_guard.iter().map(|(mac, ipv6_network)| (mac.clone(), ipv6_network.clone()) ).collect()
    read_guard.get(mac).cloned()
}

pub fn get_all_lan_ipv6_addresses() -> Vec<Ipv6Net> {
    let read_guard = LAN_IPV6_ADDRESSES_LIST.read().unwrap().clone();
    // read_guard.iter().map(|(mac, ipv6_networks)| (mac.clone(), ipv6_networks.clone()) ).collect()
    //read_guard.into_values().map(|x| x.clone()).flatten().collect()
    read_guard.values().flat_map(|x|x.clone()).collect()
}

pub fn remove_lan_ipv6_address(mac: &MacAddr) {
    let mut write_guard = LAN_IPV6_ADDRESSES_LIST.write().unwrap();
    // if let Some(ipv6_networks) = write_guard.get_mut(mac) {
    //     if let Some(index) = ipv6_networks.iter().position(|x| x == ipv6_network) {
    //         ipv6_networks.remove(index);
    //     }
    // }
    write_guard.remove_entry(mac);
}
// 清空LAN_NEIBORHOOD_IPV6中的所有IPv6网络
pub fn clear_lan_ipv6_addresses_list() {
    let mut write_guard = LAN_IPV6_ADDRESSES_LIST.write().unwrap();
    write_guard.clear();
}


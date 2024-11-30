use std::net::Ipv6Addr;

pub fn ipv6_addr_u8_to_string(u8addr: &[u8]) -> String {
    if u8addr.len() == 16 {
        let u8l16addr: [u8;16] = u8addr.try_into().expect("Convert u8addr to u8l16addr failed!");
        let addr = Ipv6Addr::from(u8l16addr);
        let addr_str = addr.to_string();
        return addr_str;
    } else {
        return "::".to_string()
    }
}
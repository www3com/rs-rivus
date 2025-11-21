#[cfg(windows)]
use ipconfig::{IfType, OperStatus};
use crate::ip::are_mac_addresses_equal_simple;

#[cfg(windows)]
pub fn get_all_self_ip() -> Option<String> {
    // 获取所有网络适配器
    let adapters = match ipconfig::get_adapters() {
        Ok(adapters) => adapters,
        Err(e) => {
            eprintln!("获取网络适配器失败: {}", e);
            return None;
        }
    };
    let mut ip_lists = Vec::new();
    // 遍历所有适配器
    for adapter in adapters {
        if adapter.oper_status() != OperStatus::IfOperStatusUp {
            continue;
        }

        // 根据接口类型判断是宽带（以太网）还是 Wi-Fi
        let interface_type_desc = match adapter.if_type() {
            IfType::EthernetCsmacd => "宽带连接 (以太网)",
            IfType::Ieee80211 => "Wi-Fi 连接",
            _ => "其他类型",
        };
        // 如果不是我们关心的类型，可以跳过
        if interface_type_desc == "其他类型" || adapter.description().contains("Virtual")
            || adapter.description().contains("Bluetooth")
            || adapter.description().contains("USB4")
            || adapter.description().contains("Remote NDIS") {
            continue;
        }


        for ip in adapter.ip_addresses() {
            if ip.is_ipv4() {
                println!("  友好名称: {}", adapter.friendly_name());
                println!("  描述: {}", adapter.description());
                let ip_str = ip.to_string();
                println!("  ip: {}", ip_str);
                println!("mac_addr{:?}", adapter.physical_address());
                ip_lists.push(ip_str);
            }
        }
    }
    if !ip_lists.is_empty() {
        return Some(ip_lists.join(","));
    }
    None
}

#[cfg(windows)]
pub fn get_self_wifi_ip_by_mac_addr(mac_addr: String) -> Option<String> {
    let adapters = match ipconfig::get_adapters() {
        Ok(adapters) => adapters,
        Err(e) => {
            eprintln!("获取网络适配器失败: {}", e);
            return None;
        }
    };
    // 遍历所有适配器
    for adapter in adapters {
        if adapter.oper_status() != OperStatus::IfOperStatusUp {
            continue;
        }
        if adapter.if_type() != IfType::Ieee80211 {
            continue;
        }
        // 如果不是我们关心的类型，可以跳过
        if adapter.description().contains("Virtual")
            || adapter.description().contains("Bluetooth")
            || adapter.description().contains("USB4")
            || adapter.description().contains("Remote NDIS") {
            continue;
        }
        if (!are_mac_addresses_equal_simple(adapter.physical_address(), &mac_addr)) {
            continue;
        }

        for ip in adapter.ip_addresses() {
            if ip.is_ipv4() {
                println!("  友好名称: {}", adapter.friendly_name());
                println!("  描述: {}", adapter.description());
                println!("mac_addr:{:?}", adapter.physical_address());
                let ip_str = ip.to_string();
                println!("  ip: {}", ip_str);
                return Some(ip_str);
            }
        }
    }
    None
}
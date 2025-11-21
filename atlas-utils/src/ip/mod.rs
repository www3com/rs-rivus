#![allow(unused)]

mod r#impl;

use r#impl::linux;
use r#impl::windows;
use anyhow::{Context, anyhow};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use get_if_addrs::get_if_addrs;


fn mac_to_u64(mac_address: &str) -> anyhow::Result<u64> {
    // 移除可能存在的分隔符（如冒号或连字符）
    let mac_clean = mac_address.replace(":", "").replace("-", "");

    // 检查清理后的字符串长度是否为12（6字节，每字节2个十六进制字符）
    if mac_clean.len() != 12 {
        return Err(anyhow!("无效的MAC地址格式: {}", mac_address));
    }

    // 尝试将十六进制字符串转换为u64
    match u64::from_str_radix(&mac_clean, 16) {
        Ok(value) => Ok(value),
        Err(e) => Err(anyhow!("转换MAC地址时出错: {}", e)),
    }
}

pub fn is_public_ip(ip_str: &str) -> anyhow::Result<bool> {
    // 将字符串解析为 IpAddr
    let ip = IpAddr::from_str(ip_str).context("Invalid IP address: {}")?;

    match ip {
        IpAddr::V4(ipv4) => Ok(is_public_ipv4(ipv4)),
        IpAddr::V6(ipv6) => Ok(is_public_ipv6(ipv6)),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    // 检查私有地址范围
    if octets[0] == 10 {
        // 10.0.0.0/8
        return false;
    }
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        // 172.16.0.0/12
        return false;
    }
    if octets[0] == 192 && octets[1] == 168 {
        // 192.168.0.0/16
        return false;
    }
    if octets[0] == 127 {
        // 127.0.0.0/8 回环地址
        return false;
    }
    if octets[0] == 0 {
        // 0.0.0.0/8
        return false;
    }
    true
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    // 检查链路本地地址 fe80::/10
    if segments[0] & 0xffc0 == 0xfe80 {
        return false;
    }
    // 检查唯一本地地址 fd00::/8 (在 fc00::/7 中)
    if segments[0] & 0xfe00 == 0xfc00 && segments[0] & 0xff00 == 0xfd00 {
        return false;
    }
    true
}

pub fn get_self_ip() -> Option<String> {
    if let Ok(if_addrs) = get_if_addrs() {
        for if_addr in if_addrs {
            // 过滤掉 lo0 接口
            if if_addr.name == "lo0" || if_addr.name == "lo" {
                continue;
            }
            if let IpAddr::V4(ip) = if_addr.ip() {
                println!("Interface: {}, IP: {}", if_addr.name, ip);
                return Some(ip.to_string());
            }
        }
    }
    None
}


/// 获取本机的局域网ip, 只包括常规的 网线和wifi 的ip
pub fn get_all_self_ip() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        windows::get_all_self_ip()
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        r#impl::get_all_self_ip()
    }
}


/// 根据某个 mac 地址获取其对应的ip 地址
pub fn get_self_wifi_ip_by_mac_addr(mac_addr: String) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        windows::get_self_wifi_ip_by_mac_addr(mac_addr)
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_self_wifi_ip_by_mac_addr(mac_addr)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        None
    }
}


/// 比较一个 Option<&[u8]> 和一个字符串形式的 MAC 地址是否相等。
///
/// # 参数
/// * `mac_bytes_opt`: `Option<&[u8]>` 格式的 MAC 地址，例如 Some(&[108, 31, 247, 72, 74, 129])。
/// * `mac_str`: 字符串格式的 MAC 地址，例如 "6c:1f:f7:48:4a:81"。
///
/// # 返回
/// 如果两者表示相同的地址，则返回 `true`，否则返回 `false`。
fn are_mac_addresses_equal_simple(mac_bytes_opt: Option<&[u8]>, mac_str: &str) -> bool {
    // 1. 从 Option 中解包字节切片。如果它是 None，那么它们不可能相等。
    let mac_bytes = match mac_bytes_opt {
        Some(bytes) => bytes,
        None => return false,
    };

    // 2. 将十六进制字符串解析成一个字节向量 (Vec<u8>)。
    //    - split(':') 按冒号分割字符串。
    //    - map(|s| u8::from_str_radix(s, 16)) 将每个部分从16进制解析为 u8。
    //    - collect() 将结果收集到一个 Result<Vec<u8>, _> 中。
    let parsed_str_bytes: Result<Vec<u8>, _> = mac_str
        .split(':')
        .map(|s| u8::from_str_radix(s, 16))
        .collect();

    // 3. 检查解析是否成功，并进行比较。
    match parsed_str_bytes {
        // 如果字符串成功解析成字节向量
        Ok(str_bytes_vec) => {
            // 直接比较两个字节切片是否相等。
            // Rust 允许直接比较 Vec<u8> 和 &[u8]。
            str_bytes_vec == mac_bytes
        }
        // 如果字符串格式无效，解析失败，则认为不相等。
        Err(_) => false,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_all_self_ip() {
        match get_all_self_ip() {
            Some(ip) => {
                println!("ip:{}", ip)
            }
            None => {}
        }
        ()
    }

    #[tokio::test]
    async fn test_get_self_wifi_ip_by_mac_addr() {
        let ip = get_self_wifi_ip_by_mac_addr("6c:1f:f7:48:4a:81".to_string()).unwrap();
        println!("ip :{}", ip);
        ()
    }
}





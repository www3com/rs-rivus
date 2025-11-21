

#[cfg(target_os = "macos")]
pub fn get_all_self_ip() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
pub fn get_self_wifi_ip_by_mac_addr(mac_addr: String) -> Option<String> {
    // TODO implement
    None
}
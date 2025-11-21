use local_ip_address::list_afinet_netifas;

pub mod linux;
pub mod windows;
pub mod macos;


pub(crate) fn get_all_self_ip() -> Option<String> {
    match list_afinet_netifas() {
        Ok(network_interfaces) => {
            if network_interfaces.is_empty() {
                return None;
            }
            let mut ip_list = Vec::new();

            for (name, ip) in network_interfaces.iter() {
                if !ip.is_ipv4() {
                    continue;
                }
                // 过滤掉lte 网卡
                if name.starts_with("enxac") {
                    continue;
                }

                // 根据接口名称判断其类型 (Wi-Fi 或以太网)
                // 这是一种基于通用命名约定的启发式方法

                // macOS 的 Wi-Fi 和以太网接口通常以 "en" 开头 (如 en0, en1)
                // 部分 Linux 发行版也使用此命名规则
                if name.starts_with("en") {
                    ip_list.push(ip.to_string());
                }
                // 传统 Linux 以太网接口以 "eth" 开头 (如 eth0)
                else if name.starts_with("eth") {
                    ip_list.push(ip.to_string());
                }
                // 传统 Linux Wi-Fi 接口以 "wlan" 或 "wlp" 开头 (如 wlan0, wlp2s0)
                else if name.starts_with("wlan") || name.starts_with("wlp") {
                    ip_list.push(ip.to_string());
                }
            }
            if !ip_list.is_empty() {
                return Some(ip_list.join(","));
            }
            None
        }
        Err(e) => {
            eprintln!("获取网络接口失败: {}", e);
            None
        }
    }
}
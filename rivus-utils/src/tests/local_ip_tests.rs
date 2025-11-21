#[cfg(test)]
mod tests {
    use crate::ip::get_all_self_ip;

    #[test]
    fn test_local_ip() {
        println!("正在获取本地网络接口 IP 地址...");

        if let Some(ip) = get_all_self_ip() {
            println!("ip:{}", ip);
        }
    }
}


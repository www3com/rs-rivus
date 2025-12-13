use std::error::Error;

pub struct DatabaseOptions {
    pub r#type: String,
    pub url: String,
    pub max_open_conns: u64, // 设置池最大连接数
    pub max_idle_conns: u64, // 设置池最大空闲数
    pub max_lifetime: u64,   // 设置连接最大生命周期
    pub timeout: u64,        // 设置连接池获取连接的超时时间
}

impl DatabaseOptions {
    pub fn new(r#type: String, url: String) -> Self {
        DatabaseOptions {
            r#type,
            url,
            max_open_conns: 10,
            max_idle_conns: 2,
            max_lifetime: 30_60,
            timeout: 10,
        }
    }
    pub fn max_open_conns(mut self, max_open_conns: u64) -> Self {
        self.max_open_conns = max_open_conns;
        self
    }

    pub fn max_idle_conns(mut self, max_idle_conns: u64) -> Self {
        self.max_idle_conns = max_idle_conns;
        self
    }
    pub fn max_lifetime(mut self, max_lifetime: u64) -> Self {
        self.max_lifetime = max_lifetime;
        self
    }
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

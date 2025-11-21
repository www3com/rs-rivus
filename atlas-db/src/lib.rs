use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::sync::OnceLock;

pub use chrono;
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: Option<u32>, // 最大连接数，默认值通常是 10
    pub min_connections: Option<u32>,
    pub connection_timeout: Option<u64>, //连接超时时间，单位为 Duration，默认值通常是 2 秒。
    pub idle_timeout: Option<u64>,
    pub max_lifetime: Option<u64>, // 连接最大生命周期，单位为 Duration，默认值通常是 8 小时。
}

static POOL: OnceLock<Pool<MySql>> = OnceLock::new();

pub async fn init(dc: DatabaseConfig) {
    let pool = MySqlPoolOptions::new()
        .max_connections(dc.max_connections.unwrap_or(10))
        .min_connections(dc.min_connections.unwrap_or(0))
        .idle_timeout(std::time::Duration::from_secs(dc.idle_timeout.unwrap_or(1)))
        .max_lifetime(std::time::Duration::from_secs(dc.max_lifetime.unwrap_or(28800)))
        .connect(&dc.url)
        .await
        .expect("Failed to create MySQL pool");
    POOL.set(pool).expect("Failed to set pool")
}

pub fn conn() -> anyhow::Result<&'static Pool<MySql>> {
    let pool = POOL
        .get()
        .ok_or_else(|| anyhow::anyhow!("Failed to get pool"))?;
    Ok(pool)
}

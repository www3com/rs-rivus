use anyhow::anyhow;
use futures::channel::mpsc;
use futures::SinkExt;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Msg {
    pub cli_id: u64,
    pub group: String,
    pub body: String,
}

// 使用 lazy_static 创建全局单例
lazy_static! {
    pub static ref CONN_MGR: Arc<Mutex<ConnectionManager>> =
        Arc::new(Mutex::new(ConnectionManager::new()));
}

pub struct ConnectionManager {
    connections: HashMap<u64, HashMap<usize, mpsc::Sender<String>>>,
    next_conn_id: usize,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            next_conn_id: 0,
        }
    }

    // 添加新连接并返回连接ID
    pub fn add_connection(&mut self, cli_id: u64, sender: mpsc::Sender<String>) -> usize {
        let conn_id = self.next_conn_id;
        self.next_conn_id += 1;

        self.connections
            .entry(cli_id)
            .or_default()
            .insert(conn_id, sender);

        conn_id
    }

    // 移除单个连接
    pub fn remove_connection(&mut self, cli_id: u64, conn_id: usize) {
        if let Some(cli_conns) = self.connections.get_mut(&cli_id) {
            cli_conns.remove(&conn_id);
            if cli_conns.is_empty() {
                self.connections.remove(&cli_id);
                tracing::info!(user_id = ?cli_id, "Removed user from connection manager");
            }
        }
    }
}


pub async fn send_message(cli_id: u64, body: String) -> anyhow::Result<()> {
    tracing::debug!("cli_id: {}, websocket channel received message body: {}", cli_id, body);
    let mut conn_mgr = CONN_MGR.lock().await;
    if let Some(cli_conns) = conn_mgr.connections.get_mut(&cli_id) {
        let mut failed_conn_ids = Vec::new();

        for (conn_id, sender) in cli_conns.iter_mut() {
            if let Err(e) = sender.send(body.clone()).await {
                tracing::error!(error = ?e, cli_id = %cli_id, conn_id = %conn_id, "Failed to send message to connection");
                failed_conn_ids.push(*conn_id);
            }
        }

        // 移除失败的连接
        for conn_id in failed_conn_ids {
            cli_conns.remove(&conn_id);
            tracing::debug!(cli_id = %cli_id, conn_id = %conn_id, "Removed failed connection");
        }

        // 如果用户没有任何连接了，清理用户
        if cli_conns.is_empty() {
            conn_mgr.connections.remove(&cli_id);
            tracing::info!(cli_id = %cli_id, "Removed Client from connection manager - no active connections");
        }
        Ok(())
    } else {
        tracing::debug!("Client not found in connection manager");
        Err(anyhow!("Client not found, client id: {}", cli_id))
    }
}
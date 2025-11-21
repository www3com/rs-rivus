use crate::conn_mgr::CONN_MGR;
use axum::body::Bytes;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures::channel::mpsc;
use futures::future::{select, BoxFuture};
use futures::FutureExt;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time;

// 定义心跳间隔时间（秒）
const PING_INTERVAL: u64 = 30;
// 定义心跳超时时间（秒）
const PING_TIMEOUT: u64 = 120;

// 处理 WebSocket 连接
pub async fn handle_connection(
    socket: WebSocket,
    cli_id: u64,
    msg_handler: Option<fn(cli_id: u64, text: Utf8Bytes) -> BoxFuture<'static, ()>>,
    close_handler: Option<fn(cli_id: u64) -> BoxFuture<'static, ()>>,
) {
    let (mut sender, receiver) = socket.split();
    let (tx, rx) = mpsc::channel(100);

    // 为 ping 任务创建一个单独的通道
    let (ping_tx, ping_rx) = mpsc::channel::<Message>(10);

    // 将发送者添加到管理器并获取连接ID
    let conn_id = {
        let mut manager = CONN_MGR.lock().await;
        manager.add_connection(cli_id, tx)
    };

    // 最后一次收到客户端消息的时间
    let last_client_activity = Arc::new(Mutex::new(Instant::now()));

    // 创建一个合并发送任务，处理来自两个通道的消息
    let sender_task = async move {
        let (text_rx, ping_rx) = (rx, ping_rx);
        let mut combined_stream =
            futures::stream::select(text_rx.map(|text| Message::Text(text.into())), ping_rx);

        while let Some(message) = combined_stream.next().await {
            if let Err(e) = sender.send(message).await {
                tracing::error!(error = ?e, "Failed to send message to client");
                break;
            }
        }
    }
        .boxed();

    // 创建心跳任务
    let ping_task = create_ping_task(
        cli_id,
        conn_id,
        close_handler,
        ping_tx,
        last_client_activity.clone(),
    );

    // 创建接收任务
    let receive_task = create_receive_task(
        receiver,
        cli_id.clone(),
        conn_id,
        msg_handler,
        close_handler,
        last_client_activity,
    );

    // 等待所有任务完成（任何一个任务结束都会导致连接关闭）
    let _ = select(
        select(sender_task, ping_task),
        receive_task,
    )
        .await;
}

// 创建心跳任务：定期发送 ping 消息
fn create_ping_task(
    cli_id: u64,
    conn_id: usize,
    close_handler: Option<fn(cli_id: u64) -> BoxFuture<'static, ()>>,
    mut ping_tx: mpsc::Sender<Message>,
    last_client_activity: Arc<Mutex<Instant>>,
) -> BoxFuture<'static, ()> {
    async move {
        let mut interval = time::interval(Duration::from_secs(PING_INTERVAL));

        loop {
            interval.tick().await;

            // 检查最后活动时间，如果超过超时时间则断开连接
            let last_activity = *last_client_activity.lock().await;
            if last_activity.elapsed() > Duration::from_secs(PING_TIMEOUT) {
                tracing::warn!(user_id = ?cli_id, "Client ping timeout, closing connection");
                break;
            }

            // 发送 ping 消息
            tracing::debug!(user_id = ?cli_id, "Sending ping");
            let ping_message = Message::Ping(Bytes::from(vec![]));
            if let Err(e) = ping_tx.send(ping_message).await {
                tracing::error!(error = ?e, "Failed to send ping, closing connection");
                break;
            }
        }

        // 心跳超时，从连接管理器中移除
        tracing::info!(user_id = ?cli_id, conn_id = ?conn_id, "Ping timeout, cleaning up");
        let mut manager = CONN_MGR.lock().await;
        manager.remove_connection(cli_id, conn_id);
        if let Some(f) = close_handler {
            f(cli_id).await;
        }
    }
        .boxed()
}

// 创建接收任务：处理来自客户端的消息
fn create_receive_task(
    mut receiver: futures::stream::SplitStream<WebSocket>,
    cli_id: u64,
    conn_id: usize,
    msg_handler: Option<fn(cli_id: u64, text: Utf8Bytes) -> BoxFuture<'static, ()>>,
    close_handler: Option<fn(cli_id: u64) -> BoxFuture<'static, ()>>,
    last_client_activity: Arc<Mutex<Instant>>,
) -> BoxFuture<'static, ()> {
    async move {
        while let Some(msg) = receiver.next().await {
            // 更新最后活动时间
            *last_client_activity.lock().await = Instant::now();

            match msg {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        tracing::debug!(message = ?text, "Received text message from client");
                        if let Some(f) = msg_handler {
                            f(cli_id, text).await;
                        }
                    }
                    Message::Binary(data) => {
                        tracing::info!(bytes = ?data.len(), "Received binary message from client");
                    }
                    Message::Close(_) => {
                        tracing::info!(cli_id = ?cli_id, "Client initiated close");
                        break;
                    }
                    Message::Pong(_) => {
                        tracing::debug!(cli_id = ?cli_id, "Received pong from client");
                    }
                    _ => {}
                },
                Err(e) => {
                    tracing::error!(error = ?e, "Error receiving message from client");
                    break;
                }
            }
        }

        // 客户端断开连接，从连接管理器中移除
        tracing::info!(cli_id = ?cli_id, conn_id = ?conn_id, "Client disconnected, cleaning up");
        let mut manager = CONN_MGR.lock().await;
        manager.remove_connection(cli_id, conn_id);
        if let Some(f) = close_handler {
            f(cli_id).await;
        }
    }
        .boxed()
}

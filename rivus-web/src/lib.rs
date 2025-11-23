
use axum::Router;
use tokio::signal;


pub struct ServeOptions {
    pub addr: Option<String>,
    pub router: Router,
}

pub async fn serve(options: ServeOptions) -> anyhow::Result<()> {
    // 启动服务器
    let addr = if let Some(addr) = options.addr {
        addr
    } else {
        "127.0.0.1:8000".to_string()
    };

    tracing::info!("Starting server at {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // 优雅关闭处理
    let server = axum::serve(listener, options.router).with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
        return Err(anyhow::anyhow!("Server error: {}", e));
    }

    tracing::info!("Server shutdown completed");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, starting graceful shutdown");
        },
    }
}

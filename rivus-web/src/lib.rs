use axum::{Router, middleware};
use tokio::signal;

pub mod result;

pub struct WebServer {
    router: Router,
    addr: String,
    i18n_path: String,
    default_locale: String,
}

impl WebServer {
    pub fn new(router: Router, addr: impl Into<String>) -> Self {
        Self {
            router,
            addr: addr.into(),
            i18n_path: "i18n".to_string(),
            default_locale: "en".to_string(),
        }
    }

    pub fn i18n_path(mut self, path: impl Into<String>) -> Self {
        self.i18n_path = path.into();
        self
    }

    pub fn default_locale(mut self, locale: impl Into<String>) -> Self {
        self.default_locale = locale.into();
        self
    }

    pub async fn run(self) -> anyhow::Result<()>  {

        tracing::info!("Starting web server at {}", self.addr);

        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        tracing::info!("⌛️ Waiting for connections...");
        tracing::info!("💡 Press Ctrl+C to stop the server");
        // 优雅关闭处理
        let server = axum::serve(listener, self.router).with_graceful_shutdown(shutdown_signal());
        if let Err(e) = server.await {
            tracing::error!("Server error: {}", e);
            return Err(anyhow::anyhow!("Server error: {}", e));
        }

        tracing::info!("Server shutdown completed");
        Ok(())
    }
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
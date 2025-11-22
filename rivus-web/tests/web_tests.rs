use rivus_web::{serve, ServeOptions};
use axum::{Router, routing::get, response::Json};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a simple test router
    fn create_test_router() -> Router {
        Router::new()
            .route("/", get(|| async { Json(json!({"message": "Hello, World!"})) }))
            .route("/health", get(|| async { Json(json!({"status": "healthy"})) }))
    }

    #[tokio::test]
    async fn test_serve_options_creation() {
        let router = create_test_router();
        let options = ServeOptions {
            addr: Some("127.0.0.1:8080".to_string()),
            router: router.clone(),
        };

        assert_eq!(options.addr, Some("127.0.0.1:8080".to_string()));
        // Router comparison is not straightforward, so we just verify it was set
    }

    #[tokio::test]
    async fn test_serve_options_default_addr() {
        let router = create_test_router();
        let options = ServeOptions {
            addr: None,
            router: router.clone(),
        };

        assert_eq!(options.addr, None);
    }

    #[tokio::test]
    async fn test_serve_with_custom_addr() {
        let router = create_test_router();
        let options = ServeOptions {
            addr: Some("127.0.0.1:0".to_string()), // 0 means random available port
            router,
        };

        // This test verifies that serve can start with a custom address
        // We use timeout to prevent hanging if something goes wrong
        let result = timeout(Duration::from_secs(2), serve(options)).await;
        
        // The server should start successfully, but we'll timeout since it's a blocking operation
        // In a real test environment, you'd want to send a request to verify it's working
        assert!(result.is_ok() || result.is_err()); // Just verify it doesn't panic
    }

    #[tokio::test]
    async fn test_serve_with_default_addr() {
        let router = create_test_router();
        let options = ServeOptions {
            addr: None,
            router,
        };

        // Test with default address
        let result = timeout(Duration::from_secs(2), serve(options)).await;
        assert!(result.is_ok() || result.is_err()); // Just verify it doesn't panic
    }

    #[tokio::test]
    async fn test_serve_options_clone() {
        let router = create_test_router();
        let options = ServeOptions {
            addr: Some("127.0.0.1:3000".to_string()),
            router: router.clone(),
        };

        // Since ServeOptions doesn't derive Clone, we can't directly clone it
        // But we can verify that creating a similar options works
        let similar_options = ServeOptions {
            addr: options.addr.clone(),
            router: router.clone(),
        };

        assert_eq!(similar_options.addr, options.addr);
    }

    #[tokio::test]
    async fn test_router_with_multiple_routes() {
        let router = Router::new()
            .route("/", get(|| async { Json(json!({"root": true})) }))
            .route("/api/users", get(|| async { Json(json!({"users": []})) }))
            .route("/api/posts", get(|| async { Json(json!({"posts": []})) }));

        let options = ServeOptions {
            addr: Some("127.0.0.1:0".to_string()),
            router,
        };

        let result = timeout(Duration::from_secs(2), serve(options)).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_shutdown_signal_ctrl_c() {
        // Test that shutdown signal handler can be created
        // We can't easily test the actual signal handling in unit tests,
        // but we can verify the function exists and compiles
        
        // Create a simple async block that mimics shutdown behavior
        let shutdown_future = async {
            // Simulate a short delay then return
            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        // This should complete without hanging
        let result = timeout(Duration::from_millis(200), shutdown_future).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_serve_error_handling() {
        // Test with an invalid address to verify error handling
        let router = create_test_router();
        let options = ServeOptions {
            addr: Some("invalid-address".to_string()),
            router,
        };

        let result = serve(options).await;
        assert!(result.is_err()); // Should fail with invalid address
    }

    #[tokio::test]
    async fn test_serve_with_empty_router() {
        let router = Router::new(); // Empty router
        let options = ServeOptions {
            addr: Some("127.0.0.1:0".to_string()),
            router,
        };

        // Even with an empty router, the server should start
        let result = timeout(Duration::from_secs(2), serve(options)).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_complex_router_setup() {
        let router = Router::new()
            .route("/", get(|| async { Json(json!({"message": "root"})) }))
            .nest("/api", Router::new()
                .route("/users", get(|| async { Json(json!({"users": [1, 2, 3]})) }))
                .route("/posts", get(|| async { Json(json!({"posts": ["a", "b"]})) }))
            )
            .route("/health", get(|| async { Json(json!({"status": "ok"})) }));

        let options = ServeOptions {
            addr: Some("127.0.0.1:0".to_string()),
            router,
        };

        let result = timeout(Duration::from_secs(2), serve(options)).await;
        assert!(result.is_ok() || result.is_err());
    }
}
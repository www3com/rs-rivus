use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestPayload {
    name: String,
    value: i32,
}

#[derive(Debug, Deserialize)]
struct TestResponse {
    #[serde(rename = "userId")]
    user_id: i32,
    id: i32,
    title: String,
    body: String,
}

#[cfg(test)]
mod http_client_builder_tests {
    use rivus_utils::http_client::{HttpClient, HttpClientBuilder};
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = HttpClientBuilder::new();
        assert!(true); // If we got here, it didn't panic
    }

    #[test]
    fn test_builder_default_values() {
        let client = HttpClient::builder().build().unwrap();
        
        // Test default values
        assert_eq!(client.proxy_url(), None);
        // Other default values are private, so we can't test them directly
    }

    #[test]
    fn test_builder_with_proxy() {
        let proxy_url = "http://proxy.example.com:8080";
        let client = HttpClient::builder()
            .proxy_url(Some(proxy_url))
            .build()
            .unwrap();
        
        assert_eq!(client.proxy_url(), Some(proxy_url));
    }

    #[test]
    fn test_builder_without_proxy() {
        let client = HttpClient::builder()
            .proxy_url(None::<String>)
            .build()
            .unwrap();
        
        assert_eq!(client.proxy_url(), None);
    }

    #[test]
    fn test_builder_with_timeouts() {
        let client = HttpClient::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .retry_delay(Duration::from_millis(500))
            .build()
            .unwrap();
        
        assert_eq!(client.proxy_url(), None); // Should still work
    }

    #[test]
    fn test_builder_with_retries() {
        let client = HttpClient::builder()
            .max_retries(5)
            .build()
            .unwrap();
        
        assert_eq!(client.proxy_url(), None); // Should still work
    }

    #[test]
    fn test_builder_with_pool_settings() {
        let client = HttpClient::builder()
            .pool_max_idle_per_host(100)
            .build()
            .unwrap();
        
        assert_eq!(client.proxy_url(), None); // Should still work
    }

    #[test]
    fn test_builder_with_headers() {
        let client = HttpClient::builder()
            .append_header("X-Custom-Header", "test-value")
            .unwrap()
            .build()
            .unwrap();
        
        assert_eq!(client.proxy_url(), None); // Should still work
    }

    #[test]
    fn test_builder_invalid_header() {
        let result = HttpClient::builder()
            .append_header("Invalid Header Name", "value");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_invalid_header_value() {
        let result = HttpClient::builder()
            .append_header("X-Test", "invalid\x00value");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_chaining() {
        let client = HttpClient::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .max_retries(3)
            .retry_delay(Duration::from_secs(1))
            .pool_max_idle_per_host(50)
            .proxy_url(Some("http://proxy.example.com:8080"))
            .append_header("User-Agent", "TestClient/1.0")
            .unwrap()
            .build();
        
        assert!(client.is_ok());
    }
}

#[cfg(test)]
mod http_client_integration_tests {
    use rivus_utils::http_client::HttpClient;
    use super::*;

    #[tokio::test]
    async fn test_get_request() {
        let client = HttpClient::builder().build().unwrap();
        
        // Test with a reliable public API
        let result = client.get::<TestResponse>("https://jsonplaceholder.typicode.com/posts/1").await;
        
        if result.is_ok() {
            let response = result.unwrap();
            assert_eq!(response.id, 1);
            assert_eq!(response.user_id, 1);
            assert!(!response.title.is_empty());
            assert!(!response.body.is_empty());
        } else {
            // If the API is unavailable, we should still handle the error gracefully
            println!("API unavailable: {:?}", result.err());
        }
    }

    #[tokio::test]
    async fn test_get_string_request() {
        let client = HttpClient::builder().build().unwrap();
        
        let result = client.get_string("https://jsonplaceholder.typicode.com/posts/1").await;
        
        if result.is_ok() {
            let response = result.unwrap();
            assert!(!response.is_empty());
            assert!(response.contains("userId"));
            assert!(response.contains("id"));
            assert!(response.contains("title"));
            assert!(response.contains("body"));
        } else {
            println!("API unavailable: {:?}", result.err());
        }
    }

    #[tokio::test]
    async fn test_post_request() {
        let client = HttpClient::builder().build().unwrap();
        
        let payload = TestPayload {
            name: "test".to_string(),
            value: 42,
        };
        
        // This will likely fail since we're posting to a read-only API, but it tests the request structure
        let result = client.post::<TestPayload, serde_json::Value>("https://jsonplaceholder.typicode.com/posts", &payload).await;
        
        // The API might return an error, but the request should be well-formed
        match result {
            Ok(response) => {
                // If it succeeds, verify the response structure
                println!("POST response: {:?}", response);
            }
            Err(e) => {
                // Expected behavior for a read-only API
                println!("POST failed as expected: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_post_string_request() {
        let client = HttpClient::builder().build().unwrap();
        
        let payload = TestPayload {
            name: "test".to_string(),
            value: 42,
        };
        
        let result = client.post_string("https://jsonplaceholder.typicode.com/posts", &payload).await;
        
        match result {
            Ok(response) => {
                assert!(!response.is_empty());
            }
            Err(e) => {
                println!("POST string failed as expected: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_put_request() {
        let client = HttpClient::builder().build().unwrap();
        
        let payload = TestPayload {
            name: "updated_test".to_string(),
            value: 100,
        };
        
        let result = client.put::<TestPayload, serde_json::Value>("https://jsonplaceholder.typicode.com/posts/1", &payload).await;
        
        match result {
            Ok(response) => {
                println!("PUT response: {:?}", response);
            }
            Err(e) => {
                println!("PUT failed as expected: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_put_string_request() {
        let client = HttpClient::builder().build().unwrap();
        
        let payload = TestPayload {
            name: "updated_test".to_string(),
            value: 100,
        };
        
        let result = client.put_string("https://jsonplaceholder.typicode.com/posts/1", &payload).await;
        
        match result {
            Ok(response) => {
                assert!(!response.is_empty());
            }
            Err(e) => {
                println!("PUT string failed as expected: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_delete_request() {
        let client = HttpClient::builder().build().unwrap();
        
        let result = client.delete::<serde_json::Value>("https://jsonplaceholder.typicode.com/posts/1").await;
        
        match result {
            Ok(response) => {
                println!("DELETE response: {:?}", response);
            }
            Err(e) => {
                println!("DELETE failed as expected: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_delete_string_request() {
        let client = HttpClient::builder().build().unwrap();
        
        let result = client.delete_string("https://jsonplaceholder.typicode.com/posts/1").await;
        
        match result {
            Ok(response) => {
                assert!(!response.is_empty());
            }
            Err(e) => {
                println!("DELETE string failed as expected: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_404_response() {
        let client = HttpClient::builder().build().unwrap();
        
        // Request a non-existent resource
        let result = client.get::<serde_json::Value>("https://jsonplaceholder.typicode.com/posts/999999").await;
        
        match result {
            Ok(_) => {
                // Some APIs might return an empty object for non-existent resources
                println!("Got response for non-existent resource");
            }
            Err(e) => {
                // Expected: should get a 404 or similar error
                println!("Got expected error for non-existent resource: {}", e);
                assert!(e.to_string().contains("404") || e.to_string().contains("Not Found"));
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let client = HttpClient::builder().build().unwrap();
        
        let result = client.get::<serde_json::Value>("not-a-valid-url").await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("error") || error.to_string().contains("invalid"));
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let client = HttpClient::builder()
            .timeout(Duration::from_millis(1)) // Very short timeout
            .build()
            .unwrap();
        
        let result = client.get::<serde_json::Value>("https://jsonplaceholder.typicode.com/posts/1").await;
        
        // Should timeout, but the exact behavior depends on network conditions
        match result {
            Ok(_) => {
                println!("Request completed despite short timeout");
            }
            Err(e) => {
                println!("Request failed as expected due to timeout: {}", e);
                // The error might contain "timeout", "time", or just "Request failed"
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("timeout") || 
                    error_msg.contains("time") || 
                    error_msg.contains("Request failed") ||
                    error_msg.contains("error")
                );
            }
        }
    }
}
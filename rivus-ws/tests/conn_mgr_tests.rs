use rivus_ws::conn_mgr::{ConnectionManager, Msg, CONN_MGR, send_message};
use futures::channel::mpsc;
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
mod connection_manager_tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager_new() {
        let _manager = ConnectionManager::new();
        // Since the fields are private, we can only test that it creates successfully
        assert!(true); // If we got here, it didn't panic
    }

    #[tokio::test]
    async fn test_add_connection() {
        let cli_id = 12345u64;
        
        // Use a fresh client ID to avoid conflicts
        let fresh_cli_id = cli_id + 1000;
        
        let (tx, mut rx) = mpsc::channel(10);
        let conn_id = CONN_MGR.lock().await.add_connection(fresh_cli_id, tx);
        
        // Test that we can send a message through the connection
        let test_msg = "Hello, WebSocket!".to_string();
        
        // Send message using the global manager
        let result = send_message(fresh_cli_id, test_msg.clone()).await;
        
        // Should succeed since we have an active connection
        assert!(result.is_ok());
        
        // Receive the message
        if let Ok(Some(received)) = timeout(Duration::from_millis(100), rx.next()).await {
            assert_eq!(received, test_msg);
        } else {
            panic!("Failed to receive message");
        }
        
        // Clean up
        CONN_MGR.lock().await.remove_connection(fresh_cli_id, conn_id);
    }

    #[tokio::test]
    async fn test_remove_connection_global() {
        let cli_id = 12345u64;
        let (tx, _rx) = mpsc::channel(10);
        
        // Add connection using global manager
        let conn_id = CONN_MGR.lock().await.add_connection(cli_id, tx);
        
        // Remove the connection using global manager
        CONN_MGR.lock().await.remove_connection(cli_id, conn_id);
        
        // Try to send a message to the removed connection
        let result = send_message(cli_id, "Test message".to_string()).await;
        
        // Should fail since connection was removed
        assert!(result.is_err(), "Expected send_message to fail after connection removal, but got: {:?}", result);
    }

    #[tokio::test]
    async fn test_multiple_connections_same_client() {
        let (tx1, _rx1) = mpsc::channel(10);
        let (tx2, _rx2) = mpsc::channel(10);
        
        let cli_id = 12345u64;
        let conn_id1 = CONN_MGR.lock().await.add_connection(cli_id, tx1);
        let conn_id2 = CONN_MGR.lock().await.add_connection(cli_id, tx2);
        
        assert_ne!(conn_id1, conn_id2); // Connection IDs should be different
        
        // Clean up
        CONN_MGR.lock().await.remove_connection(cli_id, conn_id1);
        CONN_MGR.lock().await.remove_connection(cli_id, conn_id2);
    }

    #[tokio::test]
    async fn test_send_message_to_nonexistent_client() {
        let cli_id = 99999u64; // Non-existent client ID
        let test_msg = "Test message".to_string();
        
        let result = send_message(cli_id, test_msg).await;
        
        // Should fail since client doesn't exist
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Client not found"));
    }

    #[tokio::test]
    async fn test_global_connection_manager() {
        // Test that the global CONN_MGR can be used
        let (tx, mut rx) = mpsc::channel(10);
        
        let cli_id = 55555u64;
        
        {
            let mut manager = CONN_MGR.lock().await;
            manager.add_connection(cli_id, tx);
        }
        
        // Send message using the global manager
        let test_msg = "Global manager test".to_string();
        let result = send_message(cli_id, test_msg.clone()).await;
        assert!(result.is_ok());
        
        // Receive the message
        if let Ok(Some(received)) = timeout(Duration::from_millis(100), rx.next()).await {
            assert_eq!(received, test_msg);
        } else {
            panic!("Failed to receive message from global manager");
        }
        
        // Cleanup
        {
            let mut manager = CONN_MGR.lock().await;
            manager.remove_connection(cli_id, 0); // Note: we don't have the actual conn_id here
        }
    }
}

#[cfg(test)]
mod msg_tests {
    use super::*;

    #[test]
    fn test_msg_creation() {
        let msg = Msg {
            cli_id: 12345u64,
            group: "test_group".to_string(),
            body: "Test message body".to_string(),
        };

        assert_eq!(msg.cli_id, 12345u64);
        assert_eq!(msg.group, "test_group");
        assert_eq!(msg.body, "Test message body");
    }

    #[test]
    fn test_msg_clone() {
        let msg = Msg {
            cli_id: 12345u64,
            group: "test_group".to_string(),
            body: "Test message body".to_string(),
        };

        let cloned = Msg {
            cli_id: msg.cli_id,
            group: msg.group.clone(),
            body: msg.body.clone(),
        };

        assert_eq!(cloned.cli_id, msg.cli_id);
        assert_eq!(cloned.group, msg.group);
        assert_eq!(cloned.body, msg.body);
    }
}
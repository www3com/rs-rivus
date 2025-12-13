use rivus_sqlx::db_pool::{DbPool, TRANSACTION_CONTEXT};
use rivus_sqlx::models::db_config::DatabaseOptions;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_concurrent_transactions() {
    let config = DatabaseOptions {
        r#type: "sqlite".to_string(),
        url: "sqlite::memory:".to_string(),
        max_open_conns: 10,
        max_idle_conns: 5,
        timeout: 5,
        max_lifetime: 3600,
    };

    let pool = Arc::new(DbPool::new("test_db", "sqlite", &config).await.unwrap());

    // Initialize table using execute_raw directly (no transaction context needed here)
    pool.execute_raw("CREATE TABLE IF NOT EXISTS concurrency_test (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();
    pool.execute_raw("DELETE FROM concurrency_test").await.unwrap();

    let mut handles = vec![];

    // Simulate 5 concurrent tasks
    for i in 0..5 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            // Each task MUST have its own scope
            TRANSACTION_CONTEXT.scope(RefCell::new(HashMap::new()), async move {
                // 1. Start transaction
                pool_clone.start_transaction().await.unwrap();
                
                // 2. Insert data
                let name = format!("Task-{}", i);
                let sql = format!("INSERT INTO concurrency_test (id, name) VALUES ({}, '{}')", i, name);
                pool_clone.execute_raw(&sql).await.unwrap();

                // Simulate processing delay
                sleep(Duration::from_millis(100)).await;

                // 3. Commit or Rollback
                if i % 2 == 0 {
                    pool_clone.commit_transaction().await.unwrap();
                    println!("Task {} committed", i);
                    true // Committed
                } else {
                    pool_clone.rollback_transaction().await.unwrap();
                    println!("Task {} rolled back", i);
                    false // Rolled back
                }
            }).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify results
    // Tasks 0, 2, 4 committed (3 rows)
    // Tasks 1, 3 rolled back (0 rows)
    
    // Check committed rows by deleting them and counting affected rows
    let rows = pool.execute_raw("DELETE FROM concurrency_test WHERE id % 2 = 0").await.unwrap();
    assert_eq!(rows, 3, "Should have 3 committed rows (0, 2, 4)");
    
    // Check rolled back rows
    let rows_rollback = pool.execute_raw("DELETE FROM concurrency_test WHERE id % 2 != 0").await.unwrap();
    assert_eq!(rows_rollback, 0, "Should have 0 rolled back rows");
}

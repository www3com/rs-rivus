use rivus_sqlx::db_pool::{DbPool, TRANSACTION_CONTEXT};
use rivus_sqlx::models::db_config::DatabaseOptions;
use std::collections::HashMap;
use std::cell::RefCell;

#[tokio::test]
async fn test_tls_transaction() {
    // Use shared cache for in-memory sqlite so connections share data
    let config = DatabaseOptions::new(
        "sqlite".to_string(),
        "sqlite::memory:?cache=shared".to_string(),
    );
    let pool = DbPool::new("test_tls", "sqlite", &config).await.unwrap();

    // Create table (auto-commit mode, uses pool directly)
    pool.execute_raw("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();

    // Start TLS scope
    TRANSACTION_CONTEXT.scope(RefCell::new(HashMap::new()), async {
        // Start transaction
        pool.start_transaction().await.unwrap();
        
        // Insert data in transaction
        pool.execute_raw("INSERT INTO items (id, name) VALUES (1, 'Item1')").await.unwrap();
        
        // Commit
        pool.commit_transaction().await.unwrap();
    }).await;
    
    // Verify data exists (auto-commit mode)
    // We don't have a fetch method implemented in execute_raw (it returns rows affected),
    // but we can try to delete and check affected rows.
    let rows = pool.execute_raw("DELETE FROM items WHERE id = 1").await.unwrap();
    assert_eq!(rows, 1, "Should have deleted 1 row");
}

#[tokio::test]
async fn test_tls_rollback() {
    let config = DatabaseOptions::new(
        "sqlite".to_string(),
        "sqlite::memory:?cache=shared".to_string(),
    );
    let pool = DbPool::new("test_tls_rollback", "sqlite", &config).await.unwrap();

    pool.execute_raw("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();

    TRANSACTION_CONTEXT.scope(RefCell::new(HashMap::new()), async {
        pool.start_transaction().await.unwrap();
        pool.execute_raw("INSERT INTO items (id, name) VALUES (1, 'Item1')").await.unwrap();
        pool.rollback_transaction().await.unwrap();
    }).await;

    // Verify data does NOT exist
    let rows = pool.execute_raw("DELETE FROM items WHERE id = 1").await.unwrap();
    assert_eq!(rows, 0, "Should have deleted 0 rows");
}

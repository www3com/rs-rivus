use rivus_sqlx::db_conn::{ConnManager};
use rivus_sqlx::db_pool::DbPoolInner;
use rivus_sqlx::models::db_config::DatabaseOptions;

#[tokio::test]
async fn test_db_init_get_and_close() {
    let config = DatabaseOptions::new(
        "sqlite".to_string(),
        "sqlite::memory:".to_string(),
    );
    
    // Test initialization
    let res = ConnManager::open("test_db_lifecycle", "sqlite", &config).await;
    assert!(res.is_ok(), "Failed to init db: {:?}", res.err());

    // Test retrieval
    let pool = ConnManager::by("test_db_lifecycle");
    assert!(pool.is_some(), "Failed to get db");
    
    // Test functionality
    if let Some(pool) = pool {
        if let DbPoolInner::Sqlite(pool) = &pool.inner {
            let row: (i64,) = sqlx::query_as("SELECT 1")
                .fetch_one(pool)
                .await
                .expect("Failed to execute query");
            assert_eq!(row.0, 1);
        } else {
            panic!("Retrieved db is not sqlite or wrong type");
        }
    } else {
        panic!("Failed to retrieve db");
    }

    // Test close
    let closed = ConnManager::close("test_db_lifecycle").await;
    assert!(closed, "Failed to close db");

    // Verify it's gone
    let pool_after = ConnManager::by("test_db_lifecycle");
    assert!(pool_after.is_none(), "DB should be removed after close");

    // Test double close
    let closed_again = ConnManager::close("test_db_lifecycle").await;
    assert!(!closed_again, "Double close should return false");
}

#[tokio::test]
async fn test_default_db() {
    let config = DatabaseOptions::new(
        "sqlite".to_string(),
        "sqlite::memory:".to_string(),
    );

    // Initially there should be no default db
    let pool = ConnManager::get();
    assert!(pool.is_none(), "There should be no default db initially");

    // Create a default db
    let res = ConnManager::open("default", "sqlite", &config).await;
    assert!(res.is_ok(), "Failed to init default db: {:?}", res.err());

    // Now there should be a default db
    let pool = ConnManager::get();
    assert!(pool.is_some(), "Failed to get default db");
    
    // Test functionality
    if let Some(pool) = pool {
        if let DbPoolInner::Sqlite(pool) = &pool.inner {
            let row: (i64,) = sqlx::query_as("SELECT 1")
                .fetch_one(pool)
                .await
                .expect("Failed to execute query");
            assert_eq!(row.0, 1);
        } else {
            panic!("Retrieved default db is not sqlite or wrong type");
        }
    } else {
        panic!("Failed to retrieve default db");
    }

    // Close the default db
    let closed = ConnManager::close("default").await;
    assert!(closed, "Failed to close default db");

    // Verify it's gone
    let pool_after = ConnManager::get();
    assert!(pool_after.is_none(), "Default DB should be removed after close");
}
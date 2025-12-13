use rivus_sqlx::db_conn::ConnManager;
// use rivus_sqlx::db_pool::DbPool;
use rivus_sqlx::models::db_config::DatabaseOptions;
use rivus_sqlx::orm::crud_traits::CrudRepository;
use rivus_sqlx::orm::sqlx_impl::SqlxRepository;
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestEntity {
    id: i64,
    name: String,
}

#[tokio::test]
async fn test_sqlite_repository_crud() {
    let config = DatabaseOptions::new(
        "sqlite".to_string(),
        "sqlite::memory:".to_string(),
    );
    ConnManager::open("test_repo", "sqlite", &config).await.expect("Failed to open db");
    let pool = ConnManager::by("test_repo").expect("Failed to get pool");

    // Init table
    pool.execute_raw("CREATE TABLE test_entity (id INTEGER PRIMARY KEY, name TEXT)").await.expect("Failed to create table");

    let repo = SqlxRepository;

    // Create
    let entity: TestEntity = repo.create(
        &pool,
        "INSERT INTO test_entity (id, name) VALUES (?, ?) RETURNING id, name",
        vec![Value::from(1), Value::from("Alice")]
    ).await.expect("Failed to create");
    assert_eq!(entity, TestEntity { id: 1, name: "Alice".to_string() });

    // Get
    let fetched: Option<TestEntity> = repo.get(
        &pool,
        "SELECT id, name FROM test_entity WHERE id = ?",
        vec![Value::from(1)]
    ).await.expect("Failed to get");
    assert_eq!(fetched, Some(TestEntity { id: 1, name: "Alice".to_string() }));

    // Update
    let rows = repo.update(
        &pool,
        "UPDATE test_entity SET name = ? WHERE id = ?",
        vec![Value::from("Bob"), Value::from(1)]
    ).await.expect("Failed to update");
    assert_eq!(rows, 1);

    // Verify Update
    let fetched: Option<TestEntity> = repo.get(
        &pool,
        "SELECT id, name FROM test_entity WHERE id = ?",
        vec![Value::from(1)]
    ).await.expect("Failed to get");
    assert_eq!(fetched, Some(TestEntity { id: 1, name: "Bob".to_string() }));

    // Delete
    let rows = repo.delete(
        &pool,
        "DELETE FROM test_entity WHERE id = ?",
        vec![Value::from(1)]
    ).await.expect("Failed to delete");
    assert_eq!(rows, 1);

    // Verify Delete
    let fetched: Option<TestEntity> = repo.get(
        &pool,
        "SELECT id, name FROM test_entity WHERE id = ?",
        vec![Value::from(1)]
    ).await.expect("Failed to get");
    assert_eq!(fetched, None);

    ConnManager::close("test_repo").await;
}

#[tokio::test]
async fn test_other_repository_fallback() {
    let config = DatabaseOptions::new(
        "other".to_string(),
        "other_url".to_string(),
    );
    // DbPool::new allows "other" type now
    ConnManager::open("test_other", "other_type", &config).await.expect("Failed to open db");
    let pool = ConnManager::by("test_other").expect("Failed to get pool");

    let repo = SqlxRepository;

    // Try Get
    let result: Result<Option<TestEntity>, _> = repo.get(
        &pool,
        "SELECT * FROM table",
        vec![]
    ).await;

    assert!(result.is_err());
    let err = result.err().unwrap();
    // Assuming error message contains "not implemented" or similar from OtherRepository
    assert!(format!("{}", err).contains("Unsupported database type"));

    ConnManager::close("test_other").await;
}

// use rivus_sqlx::db_pool::{DbPool, DbPoolInner, DbTransaction};
// use rivus_sqlx::models::db_config::DatabaseOptions;

// #[tokio::test]
// async fn test_transaction_commit() {
//     let config = DatabaseOptions::new(
//         "sqlite".to_string(),
//         "sqlite::memory:".to_string(),
//     );
//     let pool = DbPool::new("test_commit", "sqlite", &config).await.unwrap();

//     // Create table
//     if let DbPoolInner::Sqlite(p) = &pool.inner {
//         sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
//             .execute(p)
//             .await
//             .unwrap();
//     } else {
//         panic!("Not sqlite");
//     }

//     // Begin transaction
//     // let mut tx = pool.begin().await.unwrap();

//     // // Insert data
//     // match &mut tx {
//     //     DbTransaction::Sqlite(t) => {
//     //         sqlx::query("INSERT INTO users (id, name) VALUES (1, 'Alice')")
//     //             .execute(&mut **t)
//     //             .await
//     //             .unwrap();
//     //     }
//     //     _ => panic!("Wrong db type"),
//     // }

//     // // Commit
//     // tx.commit().await.unwrap();

//     // // Verify
//     // if let DbPoolInner::Sqlite(p) = &pool.inner {
//     //     let row: (String,) = sqlx::query_as("SELECT name FROM users WHERE id = 1")
//     //         .fetch_one(p)
//     //         .await
//     //         .unwrap();
//     //     assert_eq!(row.0, "Alice");
//     // }
// }

// #[tokio::test]
// async fn test_transaction_rollback() {
//     // ... (commented out due to missing begin() method)
// }

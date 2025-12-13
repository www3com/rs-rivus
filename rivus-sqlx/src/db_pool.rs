use crate::error::DbError;
use crate::models::db_config::DatabaseOptions;
use serde::de::DeserializeOwned;
use sqlx::pool::PoolConnection;
use sqlx::{FromRow, MySql, Pool, Postgres, Sqlite, Transaction};
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct DbPool {
    pub name: String,
    pub inner: DbPoolInner,
}

#[derive(Clone, Debug)]
pub enum DbPoolInner {
    MySql(Pool<MySql>),
    Sqlite(Pool<Sqlite>),
    Postgres(Pool<Postgres>),
    Other(String),
}

pub enum DbConnection {
    MySql(PoolConnection<MySql>),
    Sqlite(PoolConnection<Sqlite>),
    Postgres(PoolConnection<Postgres>),
}

tokio::task_local! {
    pub static TRANSACTION_CONTEXT: RefCell<HashMap<String, Arc<Mutex<DbConnection>>>>;
}

pub enum DbTransaction<'c> {
    MySql(Transaction<'c, MySql>),
    Sqlite(Transaction<'c, Sqlite>),
    Postgres(Transaction<'c, Postgres>),
}

impl<'c> DbTransaction<'c> {
    pub async fn commit(self) -> Result<(), DbError> {
        match self {
            DbTransaction::MySql(tx) => tx.commit().await?,
            DbTransaction::Sqlite(tx) => tx.commit().await?,
            DbTransaction::Postgres(tx) => tx.commit().await?,
        }
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), DbError> {
        match self {
            DbTransaction::MySql(tx) => tx.rollback().await?,
            DbTransaction::Sqlite(tx) => tx.rollback().await?,
            DbTransaction::Postgres(tx) => tx.rollback().await?,
        }
        Ok(())
    }
}

macro_rules! dispatch_db {
    ($self:expr, $conn:ident, $body:expr) => {{
        let tx_conn = TRANSACTION_CONTEXT
            .try_with(|map| map.borrow().get(&$self.name).cloned())
            .ok()
            .flatten();

        if let Some(conn_arc) = tx_conn {
            let mut conn_guard = conn_arc.lock().await;
            match &mut *conn_guard {
                DbConnection::MySql(c) => {
                    let $conn = &mut **c;
                    $body
                }
                DbConnection::Sqlite(c) => {
                    let $conn = &mut **c;
                    $body
                }
                DbConnection::Postgres(c) => {
                    let $conn = &mut **c;
                    $body
                }
            }
        } else {
            match &$self.inner {
                DbPoolInner::MySql($conn) => $body,
                DbPoolInner::Sqlite($conn) => $body,
                DbPoolInner::Postgres($conn) => $body,
                DbPoolInner::Other(_) => panic!("Direct DbPool execution not supported for 'Other' database type. Use Repository."),
            }
        }
    }};
}

impl DbPool {
    pub async fn new(name: &str, r#type: &str, config: &DatabaseOptions) -> Result<Self, DbError> {
        let inner = match r#type {
            "mysql" => Self::mysql(config).await?,
            "sqlite" => Self::sqlite(config).await?,
            "postgres" => Self::postgres(config).await?,
            _ => DbPoolInner::Other(r#type.to_string()),
        };
        Ok(Self {
            name: name.to_string(),
            inner,
        })
    }

    async fn mysql(config: &DatabaseOptions) -> Result<DbPoolInner, DbError> {
        let options = sqlx::mysql::MySqlConnectOptions::from_str(&config.url)?;
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(config.max_open_conns as u32)
            .min_connections(config.max_idle_conns as u32)
            .acquire_timeout(Duration::from_secs(config.timeout))
            .max_lifetime(Duration::from_secs(config.max_lifetime))
            .connect_with(options)
            .await?;
        Ok(DbPoolInner::MySql(pool))
    }

    async fn sqlite(config: &DatabaseOptions) -> Result<DbPoolInner, DbError> {
        let options =
            sqlx::sqlite::SqliteConnectOptions::from_str(&config.url)?.create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(config.max_open_conns as u32)
            .min_connections(config.max_idle_conns as u32)
            .acquire_timeout(Duration::from_secs(config.timeout))
            .max_lifetime(Duration::from_secs(config.max_lifetime))
            .connect_with(options)
            .await?;
        Ok(DbPoolInner::Sqlite(pool))
    }

    async fn postgres(config: &DatabaseOptions) -> Result<DbPoolInner, DbError> {
        let options = sqlx::postgres::PgConnectOptions::from_str(&config.url)?;
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_open_conns as u32)
            .min_connections(config.max_idle_conns as u32)
            .acquire_timeout(Duration::from_secs(config.timeout))
            .max_lifetime(Duration::from_secs(config.max_lifetime))
            .connect_with(options)
            .await?;
        Ok(DbPoolInner::Postgres(pool))
    }

    pub(crate) async fn close(&self) {
        match &self.inner {
            DbPoolInner::MySql(pool) => pool.close().await,
            DbPoolInner::Sqlite(pool) => pool.close().await,
            DbPoolInner::Postgres(pool) => pool.close().await,
            DbPoolInner::Other(_) => {}
        }
    }

    pub async fn start_transaction(&self) -> Result<(), DbError> {
        let conn = match &self.inner {
            DbPoolInner::MySql(p) => {
                let mut c = p.acquire().await?;
                sqlx::query("BEGIN").execute(&mut *c).await?;
                DbConnection::MySql(c)
            }
            DbPoolInner::Sqlite(p) => {
                let mut c = p.acquire().await?;
                sqlx::query("BEGIN").execute(&mut *c).await?;
                DbConnection::Sqlite(c)
            }
            DbPoolInner::Postgres(p) => {
                let mut c = p.acquire().await?;
                sqlx::query("BEGIN").execute(&mut *c).await?;
                DbConnection::Postgres(c)
            }
            DbPoolInner::Other(_) => return Err(DbError::from("Transactions not supported for Other DB types")),
        };

        TRANSACTION_CONTEXT.try_with(|map| {
            map.borrow_mut().insert(self.name.clone(), Arc::new(Mutex::new(conn)));
        }).map_err(|_| DbError::from("Transaction context not found. Ensure you are within a `TRANSACTION_CONTEXT.scope`."))?;

        Ok(())
    }

    pub async fn commit_transaction(&self) -> Result<(), DbError> {
        let conn_arc = TRANSACTION_CONTEXT
            .try_with(|map| map.borrow_mut().remove(&self.name))
            .map_err(|_| DbError::from("Transaction context not found"))?
            .ok_or_else(|| DbError::from("No active transaction to commit"))?;

        let mut conn_guard = conn_arc.lock().await;
        match &mut *conn_guard {
            DbConnection::MySql(c) => {
                sqlx::query("COMMIT").execute(&mut **c).await?;
            }
            DbConnection::Sqlite(c) => {
                sqlx::query("COMMIT").execute(&mut **c).await?;
            }
            DbConnection::Postgres(c) => {
                sqlx::query("COMMIT").execute(&mut **c).await?;
            }
        }
        Ok(())
    }

    pub async fn rollback_transaction(&self) -> Result<(), DbError> {
        let conn_arc = TRANSACTION_CONTEXT
            .try_with(|map| map.borrow_mut().remove(&self.name))
            .map_err(|_| DbError::from("Transaction context not found"))?
            .ok_or_else(|| DbError::from("No active transaction to rollback"))?;

        let mut conn_guard = conn_arc.lock().await;
        match &mut *conn_guard {
            DbConnection::MySql(c) => {
                sqlx::query("ROLLBACK").execute(&mut **c).await?;
            }
            DbConnection::Sqlite(c) => {
                sqlx::query("ROLLBACK").execute(&mut **c).await?;
            }
            DbConnection::Postgres(c) => {
                sqlx::query("ROLLBACK").execute(&mut **c).await?;
            }
        }
        Ok(())
    }

    // Helper to execute query with potential transaction
    // This is a minimal example to support "insert/update" logic
    pub async fn execute_raw(&self, sql: &str) -> Result<u64, DbError> {
        let rows_affected = dispatch_db!(self, conn, {
            sqlx::query(sql).execute(conn).await?.rows_affected()
        });
        Ok(rows_affected)
    }

    // --- CRUD with TLS support ---

    pub async fn get<T, A>(&self, sql: &str, _args: A) -> Result<Option<T>, DbError>
    where
        T: DeserializeOwned + Send + Unpin,
        T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow>,
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow>,
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow>,
        A: Send + Sync,
    {
        let res = dispatch_db!(self, conn, {
            sqlx::query_as::<_, T>(sql)
                .fetch_optional(conn)
                .await?
        });
        Ok(res)
    }

    /// 根据 SQL 和参数获取实体列表
    pub async fn list<T, A>(&self, _sql: &str, _args: A) -> Result<Vec<T>, DbError>
    where
        T: DeserializeOwned + Send,
        A: Send + Sync,
    {
        todo!()
    }

    /// 执行创建操作，并返回结果
    pub async fn create<T, A>(&self, _sql: &str, _args: A) -> Result<T, DbError>
    where
        T: DeserializeOwned + Send,
        A: Send + Sync,
    {
        todo!()
    }

    /// 批量创建操作
    pub async fn batch_create<T, A>(&self, _sql: &str, _args: Vec<A>) -> Result<Vec<T>, DbError>
    where
        T: DeserializeOwned + Send,
        A: Send + Sync,
    {
        todo!()
    }

    /// 更新操作，返回影响的行数
    pub async fn update<A>(&self, _sql: &str, _args: A) -> Result<u64, DbError>
    where
        A: Send + Sync,
    {
        todo!()
    }

    /// 删除操作，返回影响的行数
    pub async fn delete<A>(&self, _sql: &str, _args: A) -> Result<u64, DbError>
    where
        A: Send + Sync,
    {
        todo!()
    }
}

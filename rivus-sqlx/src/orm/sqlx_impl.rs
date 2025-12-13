use crate::db_pool::{DbConnection, DbPool, DbPoolInner, TRANSACTION_CONTEXT};
use crate::error::DbError;
use crate::orm::crud_traits::CrudRepository;
use crate::orm::row_de::RowDeserializer;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::{Database, Executor, IntoArguments};
use std::future::Future;

pub struct SqlxRepository;

impl CrudRepository for SqlxRepository {
    type Connection = DbPool;
    type Error = DbError;
    type Args = Vec<Value>;

    fn get<T>(
        &self,
        cnn: &Self::Connection,
        sql: &str,
        args: Self::Args,
    ) -> impl Future<Output = Result<Option<T>, Self::Error>> + Send
    where
        T: DeserializeOwned + Send,
    {
        let sql = sql.to_string();
        let cnn = cnn.clone();
        async move {
            match &cnn.inner {
                DbPoolInner::MySql(_) => execute_get_generic::<MySqlDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Sqlite(_) => execute_get_generic::<SqliteDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Postgres(_) => execute_get_generic::<PostgresDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Other(_) => Err(DbError::from("Unsupported database type")),
            }
        }
    }

    fn list<T>(
        &self,
        cnn: &Self::Connection,
        sql: &str,
        args: Self::Args,
    ) -> impl Future<Output = Result<Vec<T>, Self::Error>> + Send
    where
        T: DeserializeOwned + Send,
    {
        let sql = sql.to_string();
        let cnn = cnn.clone();
        async move {
            match &cnn.inner {
                DbPoolInner::MySql(_) => execute_list_generic::<MySqlDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Sqlite(_) => execute_list_generic::<SqliteDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Postgres(_) => execute_list_generic::<PostgresDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Other(_) => Err(DbError::from("Unsupported database type")),
            }
        }
    }

    fn create<T>(
        &self,
        cnn: &Self::Connection,
        sql: &str,
        args: Self::Args,
    ) -> impl Future<Output = Result<T, Self::Error>> + Send
    where
        T: DeserializeOwned + Send,
    {
        let sql = sql.to_string();
        let cnn = cnn.clone();
        async move {
            match &cnn.inner {
                DbPoolInner::MySql(_) => execute_create_generic::<MySqlDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Sqlite(_) => execute_create_generic::<SqliteDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Postgres(_) => execute_create_generic::<PostgresDriver, T>(&cnn, &sql, args).await,
                DbPoolInner::Other(_) => Err(DbError::from("Unsupported database type")),
            }
        }
    }

    fn batch_create<T>(
        &self,
        cnn: &Self::Connection,
        sql: &str,
        args: Vec<Self::Args>,
    ) -> impl Future<Output = Result<Vec<T>, Self::Error>> + Send
    where
        T: DeserializeOwned + Send,
    {
        let sql = sql.to_string();
        let cnn = cnn.clone();
        async move {
            let mut results = Vec::new();
            for arg in args {
                let res: T = match &cnn.inner {
                    DbPoolInner::MySql(_) => execute_create_generic::<MySqlDriver, T>(&cnn, &sql, arg).await?,
                    DbPoolInner::Sqlite(_) => execute_create_generic::<SqliteDriver, T>(&cnn, &sql, arg).await?,
                    DbPoolInner::Postgres(_) => execute_create_generic::<PostgresDriver, T>(&cnn, &sql, arg).await?,
                    DbPoolInner::Other(_) => Err(DbError::from("Unsupported database type"))?,
                };
                results.push(res);
            }
            Ok(results)
        }
    }

    fn update(
        &self,
        cnn: &Self::Connection,
        sql: &str,
        args: Self::Args,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send {
        let sql = sql.to_string();
        let cnn = cnn.clone();
        async move {
            match &cnn.inner {
                DbPoolInner::MySql(_) => execute_update_generic::<MySqlDriver>(&cnn, &sql, args).await,
                DbPoolInner::Sqlite(_) => execute_update_generic::<SqliteDriver>(&cnn, &sql, args).await,
                DbPoolInner::Postgres(_) => execute_update_generic::<PostgresDriver>(&cnn, &sql, args).await,
                DbPoolInner::Other(_) => Err(DbError::from("Unsupported database type")),
            }
        }
    }

    fn delete(
        &self,
        cnn: &Self::Connection,
        sql: &str,
        args: Self::Args,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send {
        self.update(cnn, sql, args)
    }
}

// --- 抽象驱动层 (Abstraction Layer) ---

trait SqlxDriver: Send + Sync {
    type DB: Database;

    /// 绑定参数到查询
    fn bind_arg<'q>(
        query: sqlx::query::Query<'q, Self::DB, <Self::DB as Database>::Arguments<'q>>,
        arg: Value,
    ) -> sqlx::query::Query<'q, Self::DB, <Self::DB as Database>::Arguments<'q>>;

    /// 将数据库行直接反序列化为 T
    fn from_row<T: DeserializeOwned>(row: &<Self::DB as Database>::Row) -> Result<T, DbError>;

    /// 从 DbPool 中获取特定的连接池
    fn get_pool(pool: &DbPool) -> Result<&sqlx::Pool<Self::DB>, DbError>;

    /// 从 DbConnection 中获取特定的连接
    fn get_connection(
        conn: &mut DbConnection,
    ) -> Result<&mut <Self::DB as Database>::Connection, DbError>;

    /// 获取受影响的行数
    fn get_rows_affected(result: &<Self::DB as Database>::QueryResult) -> u64;
}

struct MySqlDriver;
struct SqliteDriver;
struct PostgresDriver;

impl SqlxDriver for MySqlDriver {
    type DB = sqlx::MySql;

    fn bind_arg<'q>(
        query: sqlx::query::Query<'q, Self::DB, <Self::DB as Database>::Arguments<'q>>,
        arg: Value,
    ) -> sqlx::query::Query<'q, Self::DB, <Self::DB as Database>::Arguments<'q>> {
        match arg {
            Value::Null => query.bind(Option::<String>::None),
            Value::Bool(b) => query.bind(b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query.bind(i)
                } else if let Some(f) = n.as_f64() {
                    query.bind(f)
                } else {
                    query.bind(n.to_string())
                }
            }
            Value::String(s) => query.bind(s),
            Value::Array(a) => query.bind(Value::Array(a)),
            Value::Object(o) => query.bind(Value::Object(o)),
        }
    }

    fn from_row<T: DeserializeOwned>(row: &sqlx::mysql::MySqlRow) -> Result<T, DbError> {
        let de = RowDeserializer::new(row);
        T::deserialize(de).map_err(|e| DbError::Config(format!("反序列化错误 (Deserialization error): {}", e)))
    }

    fn get_pool(pool: &DbPool) -> Result<&sqlx::Pool<Self::DB>, DbError> {
        if let DbPoolInner::MySql(p) = &pool.inner {
            Ok(p)
        } else {
            Err(DbError::Config("连接池类型不匹配 (Pool type mismatch)".into()))
        }
    }

    fn get_connection(
        conn: &mut DbConnection,
    ) -> Result<&mut <Self::DB as Database>::Connection, DbError> {
        if let DbConnection::MySql(c) = conn {
            Ok(&mut **c)
        } else {
            Err(DbError::Config("事务类型不匹配 (Transaction type mismatch)".into()))
        }
    }

    fn get_rows_affected(result: &sqlx::mysql::MySqlQueryResult) -> u64 {
        result.rows_affected()
    }
}

impl SqlxDriver for SqliteDriver {
    type DB = sqlx::Sqlite;

    fn bind_arg<'q>(
        query: sqlx::query::Query<'q, Self::DB, <Self::DB as Database>::Arguments<'q>>,
        arg: Value,
    ) -> sqlx::query::Query<'q, Self::DB, <Self::DB as Database>::Arguments<'q>> {
        match arg {
            Value::Null => query.bind(Option::<String>::None),
            Value::Bool(b) => query.bind(b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query.bind(i)
                } else if let Some(f) = n.as_f64() {
                    query.bind(f)
                } else {
                    query.bind(n.to_string())
                }
            }
            Value::String(s) => query.bind(s),
            // Sqlite 原生不支持 JSON 绑定，转为字符串存储
            Value::Array(a) => query.bind(Value::Array(a).to_string()),
            Value::Object(o) => query.bind(Value::Object(o).to_string()),
        }
    }

    fn from_row<T: DeserializeOwned>(row: &sqlx::sqlite::SqliteRow) -> Result<T, DbError> {
        let de = RowDeserializer::new(row);
        T::deserialize(de).map_err(|e| DbError::Config(format!("反序列化错误 (Deserialization error): {}", e)))
    }

    fn get_pool(pool: &DbPool) -> Result<&sqlx::Pool<Self::DB>, DbError> {
        if let DbPoolInner::Sqlite(p) = &pool.inner {
            Ok(p)
        } else {
            Err(DbError::Config("连接池类型不匹配 (Pool type mismatch)".into()))
        }
    }

    fn get_connection(
        conn: &mut DbConnection,
    ) -> Result<&mut <Self::DB as Database>::Connection, DbError> {
        if let DbConnection::Sqlite(c) = conn {
            Ok(&mut **c)
        } else {
            Err(DbError::Config("事务类型不匹配 (Transaction type mismatch)".into()))
        }
    }

    fn get_rows_affected(result: &sqlx::sqlite::SqliteQueryResult) -> u64 {
        result.rows_affected()
    }
}

impl SqlxDriver for PostgresDriver {
    type DB = sqlx::Postgres;

    fn bind_arg<'q>(
        query: sqlx::query::Query<'q, Self::DB, <Self::DB as Database>::Arguments<'q>>,
        arg: Value,
    ) -> sqlx::query::Query<'q, Self::DB, <Self::DB as Database>::Arguments<'q>> {
        match arg {
            Value::Null => query.bind(Option::<String>::None),
            Value::Bool(b) => query.bind(b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query.bind(i)
                } else if let Some(f) = n.as_f64() {
                    query.bind(f)
                } else {
                    query.bind(n.to_string())
                }
            }
            Value::String(s) => query.bind(s),
            Value::Array(a) => query.bind(Value::Array(a)),
            Value::Object(o) => query.bind(Value::Object(o)),
        }
    }

    fn from_row<T: DeserializeOwned>(row: &sqlx::postgres::PgRow) -> Result<T, DbError> {
        let de = RowDeserializer::new(row);
        T::deserialize(de).map_err(|e| DbError::Config(format!("反序列化错误 (Deserialization error): {}", e)))
    }

    fn get_pool(pool: &DbPool) -> Result<&sqlx::Pool<Self::DB>, DbError> {
        if let DbPoolInner::Postgres(p) = &pool.inner {
            Ok(p)
        } else {
            Err(DbError::Config("连接池类型不匹配 (Pool type mismatch)".into()))
        }
    }

    fn get_connection(
        conn: &mut DbConnection,
    ) -> Result<&mut <Self::DB as Database>::Connection, DbError> {
        if let DbConnection::Postgres(c) = conn {
            Ok(&mut **c)
        } else {
            Err(DbError::Config("事务类型不匹配 (Transaction type mismatch)".into()))
        }
    }

    fn get_rows_affected(result: &sqlx::postgres::PgQueryResult) -> u64 {
        result.rows_affected()
    }
}

// --- 通用执行逻辑 (Generic Execution Logic) ---

async fn execute_get_generic<D: SqlxDriver, T>(
    pool: &DbPool,
    sql: &str,
    args: Vec<Value>,
) -> Result<Option<T>, DbError>
where
    T: DeserializeOwned + Send,
    for<'q> <D::DB as Database>::Arguments<'q>: IntoArguments<'q, D::DB>,
    for<'c> &'c mut <D::DB as Database>::Connection: Executor<'c, Database = D::DB>,
{
    let tx_conn = TRANSACTION_CONTEXT
        .try_with(|map| map.borrow().get(&pool.name).cloned())
        .ok()
        .flatten();

    let mut query = sqlx::query(sql);
    for arg in args {
        query = D::bind_arg(query, arg);
    }

    let row = if let Some(conn_arc) = tx_conn {
        let mut conn_guard = conn_arc.lock().await;
        let conn = D::get_connection(&mut *conn_guard)?;
        query.fetch_optional(conn).await?
    } else {
        let p = D::get_pool(pool)?;
        query.fetch_optional(p).await?
    };

    if let Some(row) = row {
        let t = D::from_row(&row)?;
        Ok(Some(t))
    } else {
        Ok(None)
    }
}

async fn execute_list_generic<D: SqlxDriver, T>(
    pool: &DbPool,
    sql: &str,
    args: Vec<Value>,
) -> Result<Vec<T>, DbError>
where
    T: DeserializeOwned + Send,
    for<'q> <D::DB as Database>::Arguments<'q>: IntoArguments<'q, D::DB>,
    for<'c> &'c mut <D::DB as Database>::Connection: Executor<'c, Database = D::DB>,
{
    let tx_conn = TRANSACTION_CONTEXT
        .try_with(|map| map.borrow().get(&pool.name).cloned())
        .ok()
        .flatten();

    let mut query = sqlx::query(sql);
    for arg in args {
        query = D::bind_arg(query, arg);
    }

    let rows = if let Some(conn_arc) = tx_conn {
        let mut conn_guard = conn_arc.lock().await;
        let conn = D::get_connection(&mut *conn_guard)?;
        query.fetch_all(conn).await?
    } else {
        let p = D::get_pool(pool)?;
        query.fetch_all(p).await?
    };

    let mut results = Vec::new();
    for row in rows {
        let t = D::from_row(&row)?;
        results.push(t);
    }
    Ok(results)
}

async fn execute_create_generic<D: SqlxDriver, T>(
    pool: &DbPool,
    sql: &str,
    args: Vec<Value>,
) -> Result<T, DbError>
where
    T: DeserializeOwned + Send,
    for<'q> <D::DB as Database>::Arguments<'q>: IntoArguments<'q, D::DB>,
    for<'c> &'c mut <D::DB as Database>::Connection: Executor<'c, Database = D::DB>,
{
    let opt = execute_get_generic::<D, T>(pool, sql, args).await?;
    opt.ok_or_else(|| DbError::Config("创建操作未返回行 (Create did not return a row)".into()))
}

async fn execute_update_generic<D: SqlxDriver>(
    pool: &DbPool,
    sql: &str,
    args: Vec<Value>,
) -> Result<u64, DbError>
where
    for<'q> <D::DB as Database>::Arguments<'q>: IntoArguments<'q, D::DB>,
    for<'c> &'c mut <D::DB as Database>::Connection: Executor<'c, Database = D::DB>,
{
    let tx_conn = TRANSACTION_CONTEXT
        .try_with(|map| map.borrow().get(&pool.name).cloned())
        .ok()
        .flatten();

    let mut query = sqlx::query(sql);
    for arg in args {
        query = D::bind_arg(query, arg);
    }

    let result = if let Some(conn_arc) = tx_conn {
        let mut conn_guard = conn_arc.lock().await;
        let conn = D::get_connection(&mut *conn_guard)?;
        query.execute(conn).await?
    } else {
        let p = D::get_pool(pool)?;
        query.execute(p).await?
    };
    Ok(D::get_rows_affected(&result))
}

use crate::db_pool::DbPool;
use crate::error::DbError;
use crate::orm::crud_traits::CrudRepository;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::future::Future;

pub struct OtherRepository;

impl CrudRepository for OtherRepository {
    type Connection = DbPool;
    type Error = DbError;
    type Args = Vec<Value>;

    fn get<T>(
        &self,
        _cnn: &Self::Connection,
        _sql: &str,
        _args: Self::Args,
    ) -> impl Future<Output = Result<Option<T>, Self::Error>> + Send
    where
        T: DeserializeOwned + Send,
    {
        async move {
            Err(DbError::Config("Other repository not implemented".into()))
        }
    }

    fn list<T>(
        &self,
        _cnn: &Self::Connection,
        _sql: &str,
        _args: Self::Args,
    ) -> impl Future<Output = Result<Vec<T>, Self::Error>> + Send
    where
        T: DeserializeOwned + Send,
    {
        async move {
            Err(DbError::Config("Other repository not implemented".into()))
        }
    }

    fn create<T>(
        &self,
        _cnn: &Self::Connection,
        _sql: &str,
        _args: Self::Args,
    ) -> impl Future<Output = Result<T, Self::Error>> + Send
    where
        T: DeserializeOwned + Send,
    {
        async move {
            Err(DbError::Config("Other repository not implemented".into()))
        }
    }

    fn batch_create<T>(
        &self,
        _cnn: &Self::Connection,
        _sql: &str,
        _args: Vec<Self::Args>,
    ) -> impl Future<Output = Result<Vec<T>, Self::Error>> + Send
    where
        T: DeserializeOwned + Send,
    {
        async move {
            Err(DbError::Config("Other repository not implemented".into()))
        }
    }

    fn update(
        &self,
        _cnn: &Self::Connection,
        _sql: &str,
        _args: Self::Args,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send {
        async move {
            Err(DbError::Config("Other repository not implemented".into()))
        }
    }

    fn delete(
        &self,
        _cnn: &Self::Connection,
        _sql: &str,
        _args: Self::Args,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send {
        async move {
            Err(DbError::Config("Other repository not implemented".into()))
        }
    }
}

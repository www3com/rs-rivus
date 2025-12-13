use std::future::Future;
use serde::de::DeserializeOwned;

/// 统一的 SQL 仓库特性
/// 
/// 该特性定义了基于 SQL 字符串和参数的基础数据库操作。
/// 泛型 T 通常需要实现 DeserializeOwned 以便从查询结果中反序列化。
pub trait CrudRepository {
    type Connection;
    type Error;
    type Args;

    /// 根据 SQL 和参数获取单个实体
    fn get<T>(&self, cnn: &Self::Connection, sql: &str, args: Self::Args) -> impl Future<Output = Result<Option<T>, Self::Error>> + Send
    where T: DeserializeOwned + Send;

    /// 根据 SQL 和参数获取实体列表
    fn list<T>(&self, cnn: &Self::Connection, sql: &str, args: Self::Args) -> impl Future<Output = Result<Vec<T>, Self::Error>> + Send
    where T: DeserializeOwned + Send;

    /// 执行创建操作，并返回结果
    fn create<T>(&self, cnn: &Self::Connection, sql: &str, args: Self::Args) -> impl Future<Output = Result<T, Self::Error>> + Send
    where T: DeserializeOwned + Send;

    /// 批量创建操作
    fn batch_create<T>(&self, cnn: &Self::Connection, sql: &str, args: Vec<Self::Args>) -> impl Future<Output = Result<Vec<T>, Self::Error>> + Send
    where T: DeserializeOwned + Send;

    /// 更新操作，返回影响的行数
    fn update(&self, cnn: &Self::Connection, sql: &str, args: Self::Args) -> impl Future<Output = Result<u64, Self::Error>> + Send;

    /// 删除操作，返回影响的行数
    fn delete(&self, cnn: &Self::Connection, sql: &str, args: Self::Args) -> impl Future<Output = Result<u64, Self::Error>> + Send;
}

use std::fmt::Debug;
use rivus_sqlx::sql;
// --- 基础结构定义 ---

#[derive(Debug)]
pub struct Person {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub age: u32,
}

impl Person {
    pub fn new(name: &str, age: u32) -> Self {
        Person {
            name: name.to_string(),
            age,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct SysFolder {
    id: i32,
    name: String,
}

// 模拟 Result 类型别名
type Result<T> = std::result::Result<T, String>;

// --- 用户代码区域 ---

#[derive(Debug)]
#[sql("ssss")]
pub struct FolderDao;

impl FolderDao {
    #[sql("list_user")]
    pub async fn list(person: Person, sex: i32) -> Result<Vec<SysFolder>> {
        exec!()
    }

     #[sql("list_person")]
    pub fn test(person: Person) -> Result<Vec<Person>> {
         println!("{:?}", person);
        exec!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_folder_dao_methods() {
        let p = Person::new("Alice", 30);
        let p2 = Person::new("Bob", 25);

        println!(">>> Testing FolderDao::list");
        // 调用 list
        let _ = FolderDao::list(p, 1).await;

        println!("\n>>> Testing FolderDao::test");
        // 调用 test
        let _ = FolderDao::test(p2);
    }
}

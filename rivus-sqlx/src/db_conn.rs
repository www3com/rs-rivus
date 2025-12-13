use crate::models::db_config::DatabaseOptions;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use crate::db_pool::DbPool;

static DBS: OnceLock<RwLock<HashMap<String, DbPool>>> = OnceLock::new();

pub struct ConnManager;
impl ConnManager {
    pub async fn open(
        name: &str,
        r#type: &str,
        config: &DatabaseOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pool = DbPool::new(name, r#type, config).await?;
        Self::all().write().unwrap().insert(name.to_string(), pool);
        Ok(())
    }

    fn all() -> &'static RwLock<HashMap<String, DbPool>> {
        DBS.get_or_init(|| RwLock::new(HashMap::new()))
    }

    pub fn by(name: &str) -> Option<DbPool> {
        Self::all().read().unwrap().get(name).cloned()
    }

    pub fn get() -> Option<DbPool> {
        Self::all().read().unwrap().get("default").cloned()
    }

    pub async fn close(name: &str) -> bool {
        let pool_opt = {
            let dbs = Self::all();
            let mut map = dbs.write().unwrap();
            map.remove(name)
        };

        if let Some(pool) = pool_opt {
            pool.close().await;
            true
        } else {
            false
        }
    }
}

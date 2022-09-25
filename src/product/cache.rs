use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Serialization Failed")]
    Serialization(#[from] serde_json::Error),

    #[error("Redis operation failed")]
    Redis(#[from] redis::RedisError),
}

#[derive(Clone)]
pub struct Cache {
    pub redis_conn: Arc<Mutex<redis::aio::Connection>>,
}

impl Cache {
    pub fn new(redis_conn: redis::aio::Connection) -> Self {
        Self {
            redis_conn: Arc::new(Mutex::new(redis_conn)),
        }
    }

    pub async fn set(&self, endpoint: &str, data: &str) -> Result<(), CacheError> {
        let mut redis_conn = self.redis_conn.lock().unwrap();

        redis::pipe()
            .cmd("JSON.SET")
            .arg(&[endpoint, "$", data])
            .ignore()
            .cmd("expire")
            .arg(&[endpoint, "5"])
            .ignore()
            .query_async(redis_conn.deref_mut())
            .await?;

        Ok(())
    }

    pub async fn get(&self, endpoint: &str) -> Result<Option<String>, CacheError> {
        let mut redis_conn = self.redis_conn.lock().unwrap();

        let serialized = redis::cmd("JSON.GET")
            .arg(endpoint)
            .query_async::<_, Option<String>>(redis_conn.deref_mut())
            .await?;

        Ok(serialized)
    }
}

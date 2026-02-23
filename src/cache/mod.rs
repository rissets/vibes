use anyhow::Result;
use redis::AsyncCommands;

pub struct Cache {
    client: redis::Client,
}

impl Cache {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Cache { client })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        match self.client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                let val: Option<String> = conn.get(key).await.unwrap_or(None);
                Ok(val)
            }
            Err(_) => Ok(None),
        }
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<()> {
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await {
            if let Some(ttl) = ttl_secs {
                let _: std::result::Result<(), _> = conn.set_ex(key, value, ttl).await;
            } else {
                let _: std::result::Result<(), _> = conn.set(key, value).await;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete(&self, key: &str) -> Result<()> {
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await {
            let _: std::result::Result<(), _> = conn.del(key).await;
        }
        Ok(())
    }

    pub async fn ping(&self) -> bool {
        match self.client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                let result: std::result::Result<String, _> =
                    redis::cmd("PING").query_async(&mut conn).await;
                result.is_ok()
            }
            Err(_) => false,
        }
    }
}

use crate::utility::state::app_state;
use rand::distr::SampleString;
use rand::rng;
use redis::{AsyncCommands, RedisResult};

pub async fn generate_unique_key() -> RedisResult<String> {
    let state = app_state();

    let mut conn = state.redis.as_ref().clone();

    loop {
        let key = rand::distr::Alphanumeric
            .sample_string(&mut rng(), 6)
            .to_ascii_lowercase();

        let key_final = key_prefix(&key);

        let exists: bool = conn.exists(&key_final).await?;

        if !exists {
            return Ok(key);
        }
    }
}

pub fn key_prefix(string: &String) -> String {
    const KEY_PREFIX: &str = "share:";

    format!("{}-{}", KEY_PREFIX, string)
}

use std::collections::HashMap;
use std::fmt::{Debug};
use redis::{Commands, RedisError};

#[derive(Debug, Clone)]
pub enum Cache {
    InMemory(HashMap<String, String>),
    Redis(redis::Client),
}

impl Cache {

    pub fn try_get(self: &Self, key: &String) -> Option<String> {
        match self {
            Cache::InMemory(data) => data.get(key.as_str()).map(|str| str.to_owned()),
            Cache::Redis(client) => {
                client.get_connection()
                    .ok()
                    .map(|mut conn| conn.get(key).ok())
                    .flatten()
            }
        }
    }

    pub fn add_entry(&mut self, key: String, value: String) -> Result<(), RedisError> {
        match self {
            Cache::InMemory(data) => {
                data.insert(key, value);
                Ok(())
            }
            Cache::Redis(client) => {
                let mut connection = client.get_connection()?;
                connection.set::<String, String, String>(key, value)?;
                Ok(())
            }
        }
    }
}
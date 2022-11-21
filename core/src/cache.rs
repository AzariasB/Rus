use chrono::{NaiveDateTime, Utc};
use std::collections::HashMap;
use std::fmt::Debug;

use redis::Commands;

use crate::errors::RusError;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    long_url: String,
    expires_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub enum Cache {
    InMemory(HashMap<String, CacheEntry>),
    Redis(redis::Client),
}

impl Cache {
    pub fn try_get(&self, key: &String) -> Option<String> {
        match self {
            Cache::InMemory(data) => data.get(key.as_str()).and_then(|entry| {
                let now = Utc::now().naive_utc();
                if entry.expires_at < now {
                    None
                } else {
                    Some(entry.long_url.to_owned())
                }
            }),
            Cache::Redis(client) => client
                .get_connection()
                .ok()
                .and_then(|mut conn| conn.get(key).ok()),
        }
    }

    pub fn add_entry(
        &mut self,
        key: String,
        value: String,
        expires: NaiveDateTime,
    ) -> Result<(), RusError> {
        match self {
            Cache::InMemory(data) => {
                if expires >= Utc::now().naive_utc() {
                    data.insert(
                        key,
                        CacheEntry {
                            long_url: value,
                            expires_at: expires,
                        },
                    );
                }
                Ok(())
            }
            Cache::Redis(client) => {
                let mut connection = client.get_connection()?;
                // connection.set::<String, String, String>(key, value)?;
                let now = Utc::now().naive_utc();
                let seconds = expires - now;
                let sec_usize = usize::try_from(seconds.num_seconds());
                match sec_usize {
                    Err(_) => connection.set::<String, String, String>(key, value)?,
                    Ok(secs) => connection.set_ex::<String, String, String>(key, value, secs)?,
                };
                Ok(())
            }
        }
    }
}

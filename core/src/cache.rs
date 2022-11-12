use std::error::Error;
use async_trait::async_trait;
use ::entity::redirection;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use sea_orm::{DbConn};
use entity::redirection::Model;
use crate::Query;

#[async_trait]
pub trait Cache: Debug + Clone + Sized {
    fn try_get(&self, key: String) -> Option<&redirection::Model>;

    async fn refresh(&mut self) -> Result<(), Box<dyn Error>>;
}

pub struct MemoryCache {
    pub cache: HashMap<String, redirection::Model>,
    pub db: Arc<DbConn>
}

impl MemoryCache {
    pub fn new(conn: Arc<DbConn>) -> MemoryCache {
        MemoryCache {
            cache: HashMap::new(),
            db: conn
        }
    }
}

impl Debug for MemoryCache {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.cache.fmt(f)
    }
}

impl Clone for MemoryCache {
    fn clone(&self) -> Self {
        MemoryCache {
            db: self.db.clone(),
            cache: self.cache.clone()
        }
    }
}

#[async_trait]
impl Cache for MemoryCache {
    fn try_get(&self, key: String) -> Option<&Model> {
        self.cache.get(&key)
    }

    async fn refresh(&mut self) -> Result<(), Box<dyn Error>> {
        let redirections = Query::find_all(self.db.deref()).await?;
        self.cache.clear();
        for link in redirections {
            self.cache.insert(link.short_url.to_string(), link.clone());
        }
        Result::Ok(())
    }
}
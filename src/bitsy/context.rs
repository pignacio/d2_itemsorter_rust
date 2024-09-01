use std::{collections::HashMap, fmt::Debug};

use serde::{de::DeserializeOwned, Serialize};

pub trait ContextValue: Serialize + DeserializeOwned + Debug {}
impl<T: Serialize + DeserializeOwned + Debug> ContextValue for T {}

#[derive(Debug, Clone)]
pub struct ContextKey<T: ContextValue> {
    key: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ContextValue> ContextKey<T> {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.key
    }
}

lazy_static::lazy_static! {
    pub static ref VERSION: ContextKey<u32> = ContextKey::new("version");
    pub static ref HAS_SOCKETS: ContextKey<bool> = ContextKey::new("has_sockets");
}

pub trait ContextStore {
    fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> Option<T>;
    fn set_context<T: ContextValue>(&mut self, key: &ContextKey<T>, value: T);
}

#[derive(Debug, Clone)]
pub struct ContextMap {
    map: HashMap<String, serde_json::Value>,
}

impl ContextMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl ContextStore for ContextMap {
    fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> Option<T> {
        self.map
            .get(&key.key)
            .map(|x| serde_json::from_value::<T>(x.clone()).unwrap())
    }

    fn set_context<T: ContextValue>(&mut self, key: &ContextKey<T>, value: T) {
        self.map
            .insert(key.key.clone(), serde_json::to_value(value).unwrap());
    }
}

use std::{
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
    sync::{Mutex, MutexGuard},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{item::info::ItemInfo, quality::QualityId};

pub trait ContextValue: Serialize + DeserializeOwned + Debug + 'static {}
impl<T: Serialize + DeserializeOwned + Debug + 'static> ContextValue for T {}

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
    pub static ref HAS_RUNEWORD: ContextKey<bool> = ContextKey::new("has_runeword");
    pub static ref ITEM_INFO: ContextKey<ItemInfo> = ContextKey::new("item_info");
    pub static ref QUALITY_ID: ContextKey<QualityId> = ContextKey::new("quality_id");
}

#[derive(Debug)]
pub struct ContextMap {
    map: Rc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl Clone for ContextMap {
    fn clone(&self) -> Self {
        Self {
            map: Rc::new(Mutex::new(self.lock().clone())),
        }
    }
}

impl ContextMap {
    pub fn new() -> Self {
        Self {
            map: Rc::new(Mutex::new(HashMap::new())),
        }
    }

    fn lock(&self) -> MutexGuard<HashMap<String, serde_json::Value>> {
        self.map.lock().unwrap_or_else(|err| err.into_inner())
    }

    pub fn context_reset(&self) -> ContextResetGuard {
        ContextResetGuard {
            map: self.map.clone(),
            initial: self.lock().clone(),
        }
    }

    pub fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> Option<T> {
        self.lock()
            .get(key.as_str())
            .map(|x| serde_json::from_value::<T>(x.clone()).unwrap())
    }

    pub fn set_context<T: ContextValue, V: Into<Option<T>>>(
        &self,
        key: &ContextKey<T>,
        value: V,
    ) -> Option<T> {
        let mut map = self.lock();
        let previous = if let Some(value) = value.into() {
            map.insert(
                key.as_str().to_string(),
                serde_json::to_value(value).unwrap(),
            )
        } else {
            map.remove(key.as_str())
        };
        previous.map(|json| serde_json::from_value::<T>(json).unwrap())
    }
}

#[must_use = "if unused the context will be reverted immediately"]
#[derive(Debug)]
pub struct ContextResetGuard {
    map: Rc<Mutex<HashMap<String, serde_json::Value>>>,
    initial: HashMap<String, serde_json::Value>,
}

impl Drop for ContextResetGuard {
    fn drop(&mut self) {
        let mut map = self.map.lock().unwrap_or_else(|err| err.into_inner());
        *map = self.initial.clone();
    }
}

use lazy_static::lazy_static;
use lru::LruCache;
use std::{num::NonZeroUsize, sync::Mutex};

lazy_static! {
    pub static ref CACHE: Mutex<LruCache<String, Vec<(String, usize)>>> = {
        let size = NonZeroUsize::new(100).unwrap();
        Mutex::new(LruCache::new(size))
    };
}

pub fn insert_into_cache(key: &str, value: &Vec<(String, usize)>) {
    let mut cache = CACHE.lock().unwrap();
    let key = key.to_string();
    cache.put(key, value.to_vec());
}

pub fn retrieve_from_cache(key: &str) -> Option<Vec<(String, usize)>> {
    let mut cache = CACHE.lock().unwrap();
    cache.get(key).cloned()
}
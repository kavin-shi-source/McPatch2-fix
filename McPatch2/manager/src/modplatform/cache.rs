use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::modplatform::types::*;

/// 缓存条目
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
}

/// 平台数据缓存
pub struct ModPlatformCache {
    searches: RwLock<HashMap<String, CacheEntry<Vec<ModSearchResult>>>>,
    mod_details: RwLock<HashMap<String, CacheEntry<ModSearchResult>>>,
    versions: RwLock<HashMap<String, CacheEntry<Vec<ModVersionInfo>>>>,
    download_urls: RwLock<HashMap<String, CacheEntry<DownloadUrlEntry>>>,
    fingerprint_cache: RwLock<HashMap<String, CacheEntry<Vec<UpdateFileEntry>>>>,
    ttl_searches: Duration,
    ttl_mod_detail: Duration,
    ttl_versions: Duration,
    ttl_download: Duration,
    ttl_fingerprint: Duration,
    max_entries: usize,
}

impl ModPlatformCache {
    pub fn new(
        ttl_searches: u64,
        ttl_mod_detail: u64,
        ttl_versions: u64,
        ttl_download: u64,
        ttl_fingerprint: u64,
        max_entries: usize,
    ) -> Self {
        Self {
            searches: RwLock::new(HashMap::new()),
            mod_details: RwLock::new(HashMap::new()),
            versions: RwLock::new(HashMap::new()),
            download_urls: RwLock::new(HashMap::new()),
            fingerprint_cache: RwLock::new(HashMap::new()),
            ttl_searches: Duration::from_secs(ttl_searches),
            ttl_mod_detail: Duration::from_secs(ttl_mod_detail),
            ttl_versions: Duration::from_secs(ttl_versions),
            ttl_download: Duration::from_secs(ttl_download),
            ttl_fingerprint: Duration::from_secs(ttl_fingerprint),
            max_entries,
        }
    }

    fn is_expired<T>(entry: &CacheEntry<T>, ttl: Duration) -> bool {
        entry.created_at.elapsed() >= ttl
    }

    fn evict_if_needed<K, V>(map: &mut HashMap<K, V>, max: usize)
    where
        K: std::cmp::Eq + std::hash::Hash + Clone,
    {
        if map.len() >= max {
            let keys: Vec<K> = map.keys().take(max / 2).cloned().collect();
            for k in keys {
                map.remove(&k);
            }
        }
    }

    pub fn get_searches(&self, key: &str) -> Option<Vec<ModSearchResult>> {
        let map = self.searches.read().unwrap();
        map.get(key).and_then(|entry| {
            if Self::is_expired(entry, self.ttl_searches) {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set_searches(&self, key: String, data: Vec<ModSearchResult>) {
        let mut map = self.searches.write().unwrap();
        Self::evict_if_needed(&mut map, self.max_entries);
        map.insert(
            key,
            CacheEntry {
                data,
                created_at: Instant::now(),
            },
        );
    }

    pub fn get_mod_detail(&self, key: &str) -> Option<ModSearchResult> {
        let map = self.mod_details.read().unwrap();
        map.get(key).and_then(|entry| {
            if Self::is_expired(entry, self.ttl_mod_detail) {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set_mod_detail(&self, key: String, data: ModSearchResult) {
        let mut map = self.mod_details.write().unwrap();
        Self::evict_if_needed(&mut map, self.max_entries);
        map.insert(
            key,
            CacheEntry {
                data,
                created_at: Instant::now(),
            },
        );
    }

    pub fn get_versions(&self, key: &str) -> Option<Vec<ModVersionInfo>> {
        let map = self.versions.read().unwrap();
        map.get(key).and_then(|entry| {
            if Self::is_expired(entry, self.ttl_versions) {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set_versions(&self, key: String, data: Vec<ModVersionInfo>) {
        let mut map = self.versions.write().unwrap();
        Self::evict_if_needed(&mut map, self.max_entries);
        map.insert(
            key,
            CacheEntry {
                data,
                created_at: Instant::now(),
            },
        );
    }

    pub fn get_download_url(&self, key: &str) -> Option<DownloadUrlEntry> {
        let map = self.download_urls.read().unwrap();
        map.get(key).and_then(|entry| {
            if Self::is_expired(entry, self.ttl_download) {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set_download_url(&self, key: String, data: DownloadUrlEntry) {
        let mut map = self.download_urls.write().unwrap();
        Self::evict_if_needed(&mut map, self.max_entries);
        map.insert(
            key,
            CacheEntry {
                data,
                created_at: Instant::now(),
            },
        );
    }

    pub fn get_fingerprint_cache(&self, key: &str) -> Option<Vec<UpdateFileEntry>> {
        let map = self.fingerprint_cache.read().unwrap();
        map.get(key).and_then(|entry| {
            if Self::is_expired(entry, self.ttl_fingerprint) {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set_fingerprint_cache(&self, key: String, data: Vec<UpdateFileEntry>) {
        let mut map = self.fingerprint_cache.write().unwrap();
        Self::evict_if_needed(&mut map, self.max_entries);
        map.insert(
            key,
            CacheEntry {
                data,
                created_at: Instant::now(),
            },
        );
    }
}

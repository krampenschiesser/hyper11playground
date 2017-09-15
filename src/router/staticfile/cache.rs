// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;
use std::sync::RwLock;
use std::collections::HashMap;
use response::Response;
use error::HttpError;
use std::time::Instant;

pub struct StaticFileCache {
    entry_map: RwLock<HashMap<PathBuf, CacheEntry>>,
    max_size: usize,
}


struct CacheEntry {
    eviction_policy: EvictionPolicy,
    change_detection: ChangeDetection,
    use_etag: bool,
    path: PathBuf,
    data: Vec<u8>,
    checksum: [u8; 20],
    last_touched: Instant,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ChangeDetection {
    /// use file watcher to listen for changes
    Watch,
    /// check file meta data with every request and check if it changed (updated time)
    FileInfoChange,
    /// always cached
    Never,
    /// not cached
    AlwaysRead,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EvictionPolicy {
    //evicts entry after given ms
    AfterLastAccess(usize),
    //evicts entry when cache is getting full
    WhenMaxSizeReached,
    //never evicts
    Never
}

impl Default for EvictionPolicy {
    fn default() -> Self {
        EvictionPolicy::WhenMaxSizeReached
    }
}

impl Default for StaticFileCache {
    fn default() -> Self {
        StaticFileCache { entry_map: RwLock::new(HashMap::new()), max_size: 50_000_000 }
    }
}

impl StaticFileCache {
    pub fn new() -> Self {
        StaticFileCache::default()
    }

    pub fn with_max_size(size: usize) -> Self {
        StaticFileCache { entry_map: RwLock::new(HashMap::new()), max_size: size }
    }

    pub fn get_or_load(&self, path: &PathBuf, change_detection: ChangeDetection, evction_policy: EvictionPolicy) -> Result<Response, HttpError> {
        let mut lock = self.entry_map.write().unwrap();
        let map: &mut HashMap<PathBuf, CacheEntry> = *lock;

        if map.contains_key(path) {
            if let Some(&mut entry) = map.get_mut(path) {
                entry.last_touched = Instant::now();
            }
        } else {
            let data = load_file(path, self.max_size)?;
            let data_size = data.len();
            let checksum = checksum(data.as_bytes());

            let entry = CacheEntry {
                last_touched: Instant::now(),
                path: path.clone(),
                change_detection: change_detection,
                eviction_policy: evction_policy,
                data: data,
                checksum: data,
                use_etag: true,
            };

            check_eviction_size(map, data_size, self.max_size);
        }
    }

    fn check_eviction_size(map: &mut HashMap<PathBuf, CacheEntry>, new_element_size: usize, cache_size: usize) {
        let cache_size = map.values().map(|v| v.len()).sum();

        if cache_size + new_element_size > cache_size {
            //todo evict entries
        }
    }

    fn checksum(data: &[u8]) -> [u8; 20] {
        use sha1::Sha1;

        let mut sha1 = Sha1::new();
        sha1.update(data);
        sha1.digest().bytes()
    }

    fn load_file(path: &Path, max_file_size: usize) -> Result<Vec<u8>, ::std::io::Error> {
        if !file_in_dir.exists() {
            error!("Could not find file {:?}", file_in_dir);
            return Err(HttpError::not_found(format!("File {} not found.", file_name)));
        }

        let mut data = Vec::new();
        let mut file = File::open(file_in_dir.clone())?;
        let read = file.read_to_end(&mut data)?;
        debug!("Read {} bytes from {:?}", read, file_in_dir);
    }
}
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

struct StaticFileCache {
    entry_map: RwLock<HashMap<PathBuf, CacheEntry>>,

}

struct CacheEntry {
    change_detection: ChangeDetection,
    use_etag: bool,
    path: PathBuf,
    data: Option<Vec<u8>>
}

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
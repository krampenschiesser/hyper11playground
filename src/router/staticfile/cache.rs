// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::{PathBuf, Path};
use std::sync::RwLock;
use std::collections::HashMap;
use response::Response;
use error::HttpError;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::Read;
use mime_guess::{Mime, guess_mime_type_opt};
use std::time::SystemTime;

pub struct StaticFileCache {
    entry_map: RwLock<HashMap<PathBuf, CacheEntry>>,
    max_size: usize,
}

#[derive(Debug)]
struct CacheEntry {
    eviction_policy: EvictionPolicy,
    change_detection: ChangeDetection,
    use_etag: bool,
    path: PathBuf,
    data: Vec<u8>,
    checksum: [u8; 20],
    last_touched: Instant,
    last_modification: SystemTime,
    mime_type: Option<Mime>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ChangeDetection {
    /// check file meta data with every request and check if it changed (updated time)
    FileInfoChange,
    ///read if cache entry older than time
    Timed(Duration),
    /// always cached
    Never,
    /// not cached
    NoCache,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EvictionPolicy {
    //evicts entry after given ms
    AfterLastAccess(Duration),
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

    pub fn get_or_load(&self, path: &PathBuf, change_detection: ChangeDetection, evction_policy: EvictionPolicy, etag: Option<&str>) -> Result<Response, HttpError> {
        use std::ops::DerefMut;

        let mut lock = self.entry_map.write().unwrap();
        let map: &mut HashMap<PathBuf, CacheEntry> = lock.deref_mut();

        check_eviction_time(map, Instant::now());

        if let Some(ref mut entry) = map.get_mut(path) {
            if !file_changed(&entry.path, &entry.last_modification)? {
                entry.last_touched = Instant::now();
            }
        }
        let found = if let Some(ref entry) = map.get(path) {
            match entry.change_detection {
                ChangeDetection::Timed(timeout) => {
                    let duration = Instant::now().duration_since(entry.last_touched);
                    if duration > timeout {
                        None
                    } else {
                        let v = entry.data.clone();
                        Some(create_response(v, &entry.mime_type, &entry.checksum, etag))
                    }
                }
                ChangeDetection::Never => {
                    let v = entry.data.clone();
                    Some(create_response(v, &entry.mime_type, &entry.checksum, etag))
                }
                ChangeDetection::FileInfoChange => {
                    if !file_changed(&entry.path, &entry.last_modification)? {
                        let v = entry.data.clone();
                        Some(create_response(v, &entry.mime_type, &entry.checksum, etag))
                    } else {
                        None
                    }
                }
                ChangeDetection::NoCache => None
            }
        } else { None };

        match found {
            Some(r) => r,
            None => {
                self.load_file(map, path, change_detection, evction_policy, etag)
            }
        }
    }

    fn load_file(&self, map: &mut HashMap<PathBuf, CacheEntry>, path: &PathBuf, change_detection: ChangeDetection, evction_policy: EvictionPolicy, etag: Option<&str>) -> Result<Response, HttpError> {
        let data = load_file(path, self.max_size)?;
        let data_size = data.len();
        let checksum = checksum(data.as_ref());

        let insert = check_eviction_size(map, data_size, self.max_size)
            && change_detection != ChangeDetection::NoCache;

        let mime_type_option = guess_mime_type_opt(path);

        if change_detection == ChangeDetection::NoCache {
            return create_response(data, &mime_type_option, &checksum.bytes(), etag);
        }
        let retval = data.clone();

        let modification = ::std::fs::metadata(path)?.modified()?;

        let entry = CacheEntry {
            last_touched: Instant::now(),
            last_modification: modification,
            path: path.clone(),
            change_detection: change_detection,
            eviction_policy: evction_policy,
            data: data,
            checksum: checksum.bytes(),
            mime_type: mime_type_option.clone(),
            use_etag: true,

        };
        if insert {
            map.insert(path.clone(), entry);
        }

        create_response(retval, &mime_type_option, &checksum.bytes(), etag)
    }
}

fn file_changed(path: &Path, time: &SystemTime) -> ::std::io::Result<bool> {
    let modification = ::std::fs::metadata(path)?.modified()?;
    Ok(modification != *time)
}

fn create_response(data: Vec<u8>, mime: &Option<Mime>, checksum: &[u8], etag: Option<&str>) -> Result<Response, HttpError> {
    let mut checksum_string = String::with_capacity(20);
    for byte in checksum.iter() {
        checksum_string.push_str(format!("{:02X}", byte).as_str());
    }

    if let Some(etag) = etag {
        if checksum_string == etag {
            return Ok(Response::builder().status(::http::StatusCode::NOT_MODIFIED).body(::body::Body::empty()).build()?)
        }
    }

    let mut response = Response::from(data); //fixme would be better if response is not owning vec but could just use this vec
    if let &Some(ref mime) = mime {
        response.set_header(::http::header::CONTENT_TYPE, format!("{}", mime))?;
    }
    //fixme this is ugly
    response.set_header(::http::header::ETAG, checksum_string)?;
    Ok(response)
}

fn check_eviction_size(map: &mut HashMap<PathBuf, CacheEntry>, new_element_size: usize, max_cache_size: usize) -> bool {
    let cache_size = |map: &HashMap<PathBuf, CacheEntry>| {
        let sum: usize = map.values().map(|v| v.data.len()).sum();
        sum
    };

    if cache_size(map) + new_element_size > max_cache_size {
        trace!("Cache to big to handle new file, remove all possible entries");

        let keys: Vec<PathBuf> = map.iter().filter(|e| e.1.eviction_policy == EvictionPolicy::WhenMaxSizeReached)
            .map(|e| e.0.clone()).collect();


        //fixme very basic eviciton, chekc if new file is bigger, then don't evict, sort eviction entries by distance to new file size to remove as few entries as possible
        for key in keys.iter() {
            trace!("Removing {:?} from cache because cache too big. ", key);
            map.remove(key);

            if cache_size(map) < max_cache_size {
                break;
            }
        }
    }

    cache_size(map) + new_element_size < max_cache_size
}

fn check_eviction_time(map: &mut HashMap<PathBuf, CacheEntry>, now: Instant) {
    let keys: Vec<PathBuf> = map.iter().filter(|e| {
        match e.1.eviction_policy {
            EvictionPolicy::AfterLastAccess(duration) => {
                let duration_last_touched = now.duration_since(e.1.last_touched);
                duration_last_touched > duration
            }
            _ => false
        }
    }).map(|e| e.0.clone()).collect();

    for key in keys.iter() {
        trace!("Removing {:?} from cache because entry too old. ", key);
        map.remove(key);
    }
}

fn checksum(data: &[u8]) -> ::sha1::Digest {
    use sha1::Sha1;

    let mut sha1 = Sha1::new();
    sha1.update(data);
    sha1.digest()
}

fn load_file(path: &Path, max_file_size: usize) -> Result<Vec<u8>, HttpError> {
    if !path.exists() {
        error!("Could not find file {:?}", path);
        return Err(HttpError::not_found(format!("File {:?} not found.", path.file_name())));
    }

    let mut data = Vec::new();
    let mut file = File::open(path.clone())?;
    let read = file.read_to_end(&mut data)?;
    debug!("Read {} bytes from {:?}", read, path);
    if max_file_size < read {
        Err(HttpError::bad_request(format!("File {:?} exceeds has size {} but max allowed size is {}", path, read, max_file_size)))
    } else {
        Ok(data)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn change_detection_never() {
        let cache = StaticFileCache::new();
        let buf = PathBuf::from("examples/static/index.html");
        let result = cache.get_or_load(&buf, ChangeDetection::Never, EvictionPolicy::Never, None);
        assert!(result.is_ok(), "Result not ok but {}", result.unwrap_err());
        let response = result.unwrap();
        let body = response.into_vec().unwrap();

        let needle = "<h1 class=\"MyStyle\">Hello world</h1>";
        let haystack = String::from_utf8(body).unwrap();
        assert!(haystack.contains(needle));
    }

    #[test]
    fn change_detection_file_modification() {
        let cache = StaticFileCache::new();
        let dir = TempDir::new("cachetest").unwrap();
        let path = dir.path().join("test.input");

        write_to_file(&path, "Hello world").unwrap();
        let response = cache.get_or_load(&path, ChangeDetection::FileInfoChange, EvictionPolicy::Never, None).unwrap();
        let message = String::from_utf8(response.into_vec().unwrap()).unwrap();
        assert_eq!("Hello world", message.as_str());

        write_to_file(&path, "Hello Sauerland!").unwrap();
        let response = cache.get_or_load(&path, ChangeDetection::FileInfoChange, EvictionPolicy::Never, None).unwrap();
        let message = String::from_utf8(response.into_vec().unwrap()).unwrap();
        assert_eq!("Hello Sauerland!", message.as_str());
    }

    #[test]
    fn change_detection_no_cache() {
        let cache = StaticFileCache::new();
        let buf = PathBuf::from("examples/static/index.html");
        let result = cache.get_or_load(&buf, ChangeDetection::NoCache, EvictionPolicy::Never, None);

        assert!(result.is_ok(), "Result not ok but {}", result.unwrap_err());
        let r = cache.entry_map.read().unwrap();
        let o = r.get(&buf);
        assert!(o.is_none());
    }

    #[test]
    fn change_detection_timed() {
        let cache = StaticFileCache::new();
        let dir = TempDir::new("cachetest").unwrap();
        let path = dir.path().join("test.input");

        write_to_file(&path, "Hello world").unwrap();
        let response = cache.get_or_load(&path, ChangeDetection::Timed(Duration::from_millis(1000)), EvictionPolicy::Never, None).unwrap();
        let message = String::from_utf8(response.into_vec().unwrap()).unwrap();
        assert_eq!("Hello world", message.as_str());

        write_to_file(&path, "Hello Sauerland!").unwrap();
        let response = cache.get_or_load(&path, ChangeDetection::Timed(Duration::from_millis(1000)), EvictionPolicy::Never, None).unwrap();
        let message = String::from_utf8(response.into_vec().unwrap()).unwrap();
        assert_eq!("Hello world", message.as_str());

        {
            let mut r = cache.entry_map.write().unwrap();
            if let Some(entry) = r.get_mut(&path) {
                entry.last_touched = entry.last_touched - Duration::from_millis(1002);
            }
        }
        let response = cache.get_or_load(&path, ChangeDetection::Timed(Duration::from_millis(1000)), EvictionPolicy::Never, None).unwrap();
        let message = String::from_utf8(response.into_vec().unwrap()).unwrap();
        assert_eq!("Hello Sauerland!", message.as_str());
    }

    #[test]
    fn eviction_size() {
        let _ = ::env_logger::init();
        let cache = StaticFileCache::with_max_size(422);

        let buf = PathBuf::from("examples/static/index.html");
        let result = cache.get_or_load(&buf, ChangeDetection::Never, EvictionPolicy::WhenMaxSizeReached, None);
        assert!(result.is_ok(), "Result not ok but {}", result.unwrap_err());

        let buf = PathBuf::from("examples/static/style/style.css");
        let result = cache.get_or_load(&buf, ChangeDetection::Never, EvictionPolicy::WhenMaxSizeReached, None);
        assert!(result.is_ok(), "Result not ok but {}", result.unwrap_err());

        let map = cache.entry_map.read().unwrap();
        let size = map.len();
        assert_eq!(1, size);
        assert!(map.get(&buf).is_some());
    }

    #[test]
    fn eviction_time() {
        let _ = ::env_logger::init();
        let cache = StaticFileCache::new();

        let buf = PathBuf::from("examples/static/index.html");
        let result = cache.get_or_load(&buf, ChangeDetection::Never, EvictionPolicy::AfterLastAccess(Duration::from_millis(100)), None);
        assert!(result.is_ok(), "Result not ok but {}", result.unwrap_err());

        {
            let mut r = cache.entry_map.write().unwrap();
            if let Some(entry) = r.get_mut(&buf) {
                entry.last_touched = entry.last_touched - Duration::from_millis(110);
            }
        }

        let buf = PathBuf::from("examples/static/style/style.css");
        let result = cache.get_or_load(&buf, ChangeDetection::Never, EvictionPolicy::Never, None);
        assert!(result.is_ok(), "Result not ok but {}", result.unwrap_err());

        let map = cache.entry_map.read().unwrap();
        let size = map.len();
        assert_eq!(1, size);
        assert!(map.get(&buf).is_some());
    }

    #[test]
    fn etag() {
        let cache = StaticFileCache::new();
        let buf = PathBuf::from("examples/static/index.html");

        let result = cache.get_or_load(&buf, ChangeDetection::Never, EvictionPolicy::Never, None);
        assert!(result.is_ok(), "Result not ok but {}", result.unwrap_err());
        let response: Response = result.unwrap();
        let etag_from_response = response.headers().get(::http::header::ETAG).unwrap();
        let etag = etag_from_response.to_str().unwrap();

        let response = cache.get_or_load(&buf, ChangeDetection::Never, EvictionPolicy::Never, Some(etag)).unwrap();
        assert_eq!(::http::StatusCode::NOT_MODIFIED, response.status());
    }

    fn write_to_file(path: &PathBuf, content: &str) -> ::std::io::Result<()> {
        use std::io::Write;

        let mut f = File::create(path)?;
        f.write_all(content.as_bytes())?;
        Ok(())
    }
}
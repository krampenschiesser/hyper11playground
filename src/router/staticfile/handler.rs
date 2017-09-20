// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use std::path::PathBuf;

use handler::Handler;
use response::Response;
use error::HttpError;
use request::Request;
use super::cache::{StaticFileCache, EvictionPolicy, ChangeDetection};
use std::sync::Arc;

pub struct StaticFileHandler {
    path: PathBuf,
    cache: Arc<StaticFileCache>,
    eviction_policy: EvictionPolicy,
    change_detection: ChangeDetection,

}

impl StaticFileHandler {
    pub fn new<T: Into<PathBuf>>(path: T, cache: Arc<StaticFileCache>, eviction_policy: EvictionPolicy, change_detection: ChangeDetection) -> Self {
        StaticFileHandler { path: path.into(), cache, eviction_policy, change_detection }
    }
}

impl Handler for StaticFileHandler {
    fn handle(&self, req: &mut Request) -> Result<Response, HttpError> {
        let o = req.header(&::http::header::ETAG);

        if self.path.is_dir() {
            let mut file_in_dir = self.path.clone();

            let file_name = req.param("file").ok_or(HttpError::not_found("No file parameter found"))?;

            file_in_dir.push(file_name);

            self.cache.get_or_load(&file_in_dir, self.change_detection, self.eviction_policy, o)
        } else {
            self.cache.get_or_load(&self.path, self.change_detection, self.eviction_policy,o )
        }
    }
}

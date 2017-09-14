// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use std::path::PathBuf;
use std::fs::File;

use handler::Handler;
use response::Response;
use error::HttpError;
use request::Request;

pub struct StaticFileHandler {
    path: PathBuf,
}

impl StaticFileHandler {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        StaticFileHandler { path: path.into() }
    }
}

impl Handler for StaticFileHandler {
    fn handle(&self, req: &mut Request) -> Result<Response, HttpError> {
        use std::io::Read;

        if self.path.is_dir() {
            let mut file_in_dir = self.path.clone();

            let file_name = req.param("file").ok_or(HttpError::not_found("No file parameter found"))?;

            file_in_dir.push(file_name);

            if !file_in_dir.exists() {
                error!("Could not find file {:?}", file_in_dir);
                return Err(HttpError::not_found(format!("File {} not found.", file_name)));
            }

            let mut data = Vec::new();
            let mut file = File::open(file_in_dir.clone())?;
            let read = file.read_to_end(&mut data)?;
            debug!("Read {} bytes from {:?}", read, file_in_dir);
        } else {}
        Err(HttpError::not_found(format!("File not found.")))
    }
}

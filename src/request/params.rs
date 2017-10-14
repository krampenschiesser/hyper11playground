// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Params {
    data: BTreeMap<String, String>
}

impl From<::route_recognizer::Params> for Params {
    fn from(params: ::route_recognizer::Params) -> Self {
        let mut data: BTreeMap<String, String> = BTreeMap::new();
        for (key, value) in params.iter() {
            data.insert(key.into(), value.into());
        }
        Params { data }
    }
}

impl Params {
    pub fn new() -> Self {
        Params::default()
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
    
    pub fn inner_mut(&mut self) -> &mut BTreeMap<String,String> {
        &mut self.data
    }
}

impl Default for Params {
    fn default() -> Self {
        Params { data: BTreeMap::new() }
    }
}
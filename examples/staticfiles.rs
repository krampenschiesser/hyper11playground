// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate rest_in_rust;
extern crate env_logger;

use rest_in_rust::*;
use std::path::Path;

fn main() {
    let _ = env_logger::init();
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.static_file("/index.html", Path::new("examples/static/index.html"), ChangeDetection::FileInfoChange);
    r.static_folder("/style", Path::new("examples/static/style"), ChangeDetection::FileInfoChange);
    r.get("/hello/:world", hello_world);

    let s = Server::new(addr, r);
    s.start_http();
}
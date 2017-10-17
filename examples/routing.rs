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

fn params(req: &mut Request) -> Result<Response, HttpError> {
    let hello_str = req.param("hello").unwrap_or("Hallo ");
    let world_str = req.param("world").unwrap_or("Sauerland!");
    Ok(format!("{}{}", hello_str, world_str).into())
}
fn say(req: &mut Request) -> Result<Response, HttpError> {
    let world_str = req.param("text").unwrap_or("Silence!");
    Ok(world_str.into())
}
fn main() {
    let _ = env_logger::init();
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.get("/:hello/:world", params);
    r.get("/say/*text", say);

    let s = Server::new(addr, r);
    s.start_http();
}
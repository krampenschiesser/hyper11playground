// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate env_logger;
extern crate rest_in_rust;

use rest_in_rust::*;
use std::sync::RwLock;

struct State {
    echo_history: RwLock<Vec<String>>,
}

impl Default for State {
    fn default() -> Self {
        State { echo_history: RwLock::new(Vec::new()) }
    }
}


fn show_history(req: &mut Request) -> Result<Response, HttpError> {
    let state: &State = req.get_state().unwrap();
    let r = state.echo_history.read().unwrap();
    Ok(r.join("\n").into())
}

fn response(req: &mut Request) -> Result<Response, HttpError> {
    let o = req.param("hello");

    let state: &State = req.get_state().unwrap();
    match o {
        Some(string) => {
            let mut lock = state.echo_history.write().unwrap();
            lock.push(string.to_string());
            Ok(string.into())
        }
        None => Ok("Please provide a path parameter.".into())
    }
}

fn main() {
    let _ = env_logger::init();
    let addr = "127.0.0.1:8091".parse().unwrap();
    let state = State::default();

    let mut r = Router::new();
    r.get("/:hello", response);
    r.get("/history", show_history);

    let s = Server::new(addr, r);
    s.add_state(state);
    s.start_http();
}
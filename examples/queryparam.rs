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

fn query_param(req: &mut Request) -> Result<Response, HttpError> {
    let all_params = req.query_all();
    let mut retval = String::new();
    for (key, value) in all_params.iter() {
        retval.push_str(format!("{}\n",key).as_str());
        for string in value.iter() {
            retval.push_str(format!("\t{}\n",string).as_str());
        }
    }
    Ok(retval.into())
}

fn main() {
    let _ = env_logger::init();
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.get("/", query_param);


    let s = Server::new(addr,r);
    s.start_http();
}
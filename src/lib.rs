// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![feature(try_from)]

extern crate url;
extern crate route_recognizer;
extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate log;
extern crate native_tls;
extern crate tokio_tls;
extern crate tokio_service;
extern crate tokio_proto;
extern crate state;

extern crate http;

extern crate tokio_io;
extern crate bytes;
extern crate httparse;

#[cfg(test)]
extern crate spectral;


pub mod server;
pub mod router;
pub mod request;
pub mod response;
pub mod handler;
pub mod error;
pub mod traits;
pub mod body;

pub use router::Router;
pub use error::HttpError;
pub use handler::Handler;
pub use request::Request;
pub use response::Response;
pub use server::{Server};
pub use server::tester::ServerTester;
pub use traits::{FromRequest, FromRequestAsRef};
pub use body::{Body};

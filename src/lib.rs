// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Rest in Rust 
//!
//!
//! Rest in rust is a very basic http 1 rest server based on
//! [Tokio](https://tokio.rs/) and the [http](https://github.com/carllerche/http) crate.
//! 
//! It's main goal is to provide a good developer experience for easy development.
//! This is done by returning results from handlers and a lot of nice conversions
//! between types.
//! Everything is included, there are no plugins you have to find to *extend* behavior in order to get simple things done.
//! 
//! 
//! ## Fetures
//! 
//! * https
//! * simple routing
//! * JSON(serde) parsing in both ways (from body, to body)
//! * query params
//! * route params
//! * static file serving
//! 
//! 
//! ## Secrutiy
//! 
//! Not much about security in this crate,
//! I would not recommend it for production use as standalone,
//! always put it behind a reverse proxy.
//! However any suggestions on how to improve it are very welcome.
 
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(try_from)]
#![feature(conservative_impl_trait)]

extern crate url;
extern crate route_recognizer;
extern crate serde;
extern crate serde_json;
#[allow(unused)]
#[macro_use]
#[allow(unused)]
extern crate serde_derive;
extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate log;
extern crate native_tls;
extern crate tokio_tls;
extern crate tokio_service;
extern crate tokio_proto;
extern crate state;
extern crate mime_sniffer;
extern crate mime_guess;
extern crate http;
extern crate tokio_io;
extern crate bytes;
extern crate httparse;
extern crate sha1;
#[cfg(test)]
extern crate spectral;
#[cfg(test)]
extern crate tempdir;
#[cfg(test)]
extern crate env_logger;


pub mod server;
pub mod router;
pub mod request;
pub mod response;
pub mod handler;
pub mod error;
pub mod traits;
pub mod body;

pub use router::{Router,ChangeDetection,EvictionPolicy};
pub use error::HttpError;
pub use handler::Handler;
pub use request::Request;
pub use response::{ResponseBuilder,Response};
pub use server::Server;
pub use server::tester::ServerTester;
pub use traits::{FromRequest, FromRequestAsRef};
pub use body::Body;

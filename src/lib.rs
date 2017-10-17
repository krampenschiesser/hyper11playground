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
//! ## Features
//! 
//! ### Https
//! 
//! ```rust,no_run
//! extern crate rest_in_rust;
//! extern crate native_tls;
//! 
//! use rest_in_rust::*;
//! use native_tls::Pkcs12;
//! 
//! fn hello_world(_: &mut Request) -> Result<Response, HttpError> {
//!     Ok("hello encryption".into())
//! }
//! 
//! fn main() {
//!     let addr = "127.0.0.1:8091".parse().unwrap();
//!     let mut r = Router::new();
//!     r.get("/", hello_world);
//!     let  s = Server::new(addr,r);
//! 
//!     let der = include_bytes!("../examples/certificate.p12");
//!     let cert = Pkcs12::from_der(der, "password").unwrap();
//!     s.start_https(cert);
//! }
//! ```
//! 
//! ### Simple Routing
//! 
//! ```rust,no_run
//! extern crate rest_in_rust;
//! use rest_in_rust::*;
//! 
//! fn params(req: &mut Request) -> Result<Response, HttpError> {
//!     let hello_str = req.param("hello").unwrap_or("Hallo ");
//!     let world_str = req.param("world").unwrap_or("Sauerland!");
//!     Ok(format!("{}{}", hello_str, world_str).into())
//! }
//! fn say(req: &mut Request) -> Result<Response, HttpError> {
//!     let world_str = req.param("text").unwrap_or("Silence!");
//!     Ok(world_str.into())
//! }
//! fn main() {
//!     let addr = "127.0.0.1:8091".parse().unwrap();
//! 
//!     let mut r = Router::new();
//!     r.get("/:hello/:world", params);
//!     r.get("/say/*text", say);
//! 
//!     let s = Server::new(addr, r);
//!     s.start_http();
//! }
//! ```
//! 
//! ## JSON(serde) parsing in both ways (from body, to body)
//! 
//! ```rust,no_run
//! extern crate rest_in_rust;
//! #[macro_use]
//! extern crate serde_derive;
//! 
//! use rest_in_rust::*;
//! 
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Hello {
//!     world: String,
//! }
//! fn post_json(req: &mut Request) -> Result<Response, HttpError> {
//!     let obj: Hello = req.body().to_json()?;
//! 
//!     Ok(format!("{:?}", obj).into())
//! }
//! fn get_json(_: &mut Request) -> Result<Response, HttpError> {
//!     let obj = Hello { world: "Hello Sauerland".into() };
//!     Response::try_from_json(obj)
//! }
//! fn main() {
//!     let addr = "127.0.0.1:8091".parse().unwrap();
//! 
//!     let mut r = Router::new();
//!     r.post("/", post_json);
//!     r.get("/", get_json);
//! 
//!     let s = Server::new(addr, r);
//!     s.start_http();
//! }
//! ```
//! 
//! ## Query Params
//! 
//! ```rust,no_run
//! extern crate rest_in_rust;
//! 
//! use rest_in_rust::*;
//! 
//! fn query_param(req: &mut Request) -> Result<Response, HttpError> {
//!     let all_params = req.query_all();
//!     let mut retval = String::new();
//!     for (key, value) in all_params.iter() {
//!         retval.push_str(format!("{}\n",key).as_str());
//!         for string in value.iter() {
//!             retval.push_str(format!("\t{}\n",string).as_str());
//!         }
//!     }
//!     Ok(retval.into())
//! }
//! 
//! fn main() {
//!     let addr = "127.0.0.1:8091".parse().unwrap();
//! 
//!     let mut r = Router::new();
//!     r.get("/", query_param);
//! 
//!     let s = Server::new(addr,r);
//!     s.start_http();
//! }
//! ```
//! 
//! ## Static File Serving
//! 
//! ```rust,no_run
//! extern crate rest_in_rust;
//! 
//! use rest_in_rust::*;
//! use std::path::Path;
//! 
//! fn main() {
//!     let addr = "127.0.0.1:8091".parse().unwrap();
//!  
//!     let mut r = Router::new();
//!     r.static_path_cached("/index.html", Path::new("examples/static/index.html"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
//!     r.static_path_cached("/style", Path::new("examples/static/style"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
//! 
//!     let s = Server::new(addr, r);
//!     s.start_http();
//! } 
//! ```
//! 
//! ### Testing
//! 
//! ```
//! extern crate rest_in_rust;
//! extern crate http;
//! 
//! use std::str::FromStr;
//! use rest_in_rust::*;
//! use http::request::Builder as RequestBuilder;
//! use http::Uri;
//! 
//! fn hello_world(req: &mut Request) -> Result<Response, HttpError> {
//!     Ok(req.param("world").unwrap_or("sauerland").into())
//! }
//! fn main() {
//!     let addr = "127.0.0.1:8091".parse().unwrap();
//! 
//!     let mut r = Router::new();
//!     r.get("/hello/:world", hello_world);
//! 
//!     let s = Server::new(addr, r);
//!     let tester = s.start_testing();
//! 
//!     let request = RequestBuilder::new().uri(Uri::from_str("/hello/huhu").unwrap()).body(Body::empty()).unwrap();
//!     let response = tester.handle(request);
//!     let answer_string = response.body().to_string().unwrap();
//! 
//!     assert_eq!(200, response.status().as_u16());
//!     assert_eq!("huhu", answer_string);
//! }
//! ```
//! 
//! 
//! ## Security
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
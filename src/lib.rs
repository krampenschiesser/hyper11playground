#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![feature(try_from)]
extern crate url;
extern crate route_recognizer;
extern crate serde;
extern crate serde_json;
extern crate futures;
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
//#[cfg(debug_assertions)]
//extern crate reqwest;


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
pub use traits::{FromRequest, FromRequestAsRef};
pub use body::{Body};

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate hyper;
extern crate url;
extern crate route_recognizer;
extern crate serde;
extern crate serde_json;
extern crate futures;
#[macro_use]
extern crate log;
//#[cfg(test)]
extern crate env_logger;
extern crate native_tls;
extern crate tokio_tls;
extern crate tokio_service;
extern crate tokio_proto;

pub mod server;
pub mod router;
pub mod request;
pub mod response;
pub mod handler;
pub mod error;
pub mod prelude;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

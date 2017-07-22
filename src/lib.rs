#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate hyper;
extern crate url;
extern crate route_recognizer;
extern crate serde;
extern crate serde_json;
extern crate futures;

pub mod server;
pub mod router;
pub mod request;
pub mod response;
pub mod handler;
pub mod error;
pub mod prelude;

pub trait TResponse {
    fn get_status(&self) -> u16;
    fn get_msg(&self) -> &str;
    fn get_headers(&self) -> &::hyper::Headers;
}





#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

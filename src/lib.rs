extern crate hyper;
extern crate url;
extern crate route_recognizer;
extern crate serde;
extern crate serde_json;

use ::std::error::Error;

mod router;
pub mod prelude;

pub struct Request {}

pub struct Response {}

pub trait Handler: Send + Sync + 'static {
    fn handle(&self, req: &mut Request) -> Result<Response, HttpError>;
}

pub struct HttpError {
    status: u16,
    msg: String
}

impl<F> Handler for F
    where F: Send + Sync + 'static + Fn(&mut Request) -> Result<Response, HttpError> {
    fn handle(&self, req: &mut Request) -> Result<Response, HttpError> {
        (*self)(req)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

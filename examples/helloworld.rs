extern crate rest_in_rust;

use rest_in_rust::prelude::*;

fn hello_world(req: &mut Request) -> Result<Response, HttpError> {
    Ok(req.param("world").unwrap_or("sauerland").into())
}

fn main() {
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.get("/hello/:world", hello_world);

    let  s = Server::new(addr,r);
    s.start_http().unwrap();
}
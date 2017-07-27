extern crate hyper11playground;

use hyper11playground::prelude::*;

fn hello_world(req: &mut Request) -> Result<Response, HttpError> {
    Ok(req.param("world").unwrap_or("sauerland").into())
}

fn main() {
    let mut r = Router::new();

    r.get("/hello/:world", hello_world);

    let mut s = Server::http("127.0.0.1:8091".parse().unwrap());
    s.router(r);

    s.start();
}
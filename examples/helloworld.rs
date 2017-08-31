extern crate rest_in_rust;
extern crate env_logger;

use rest_in_rust::*;

fn hello_world(req: &mut Request) -> Result<Response, HttpError> {
    Ok(req.param("world").unwrap_or("sauerland").into())
}

fn main() {
    let _ = env_logger::init();
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.get("/hello/:world", hello_world);

    let s = Server::new(addr, r);
    s.start_http();
}
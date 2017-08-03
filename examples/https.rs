extern crate hyper11playground;
extern crate native_tls;

use hyper11playground::prelude::*;
use native_tls::Pkcs12;


fn hello_world(req: &mut Request) -> Result<Response, HttpError> {
    Ok("hello encryption".into())
}

fn main() {
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.get("/", hello_world);

    let  s = Server::new(addr,r);

    let der = include_bytes!("certificate.p12");
    let cert = Pkcs12::from_der(der, "password").unwrap();
    s.start_https(cert).unwrap();
}
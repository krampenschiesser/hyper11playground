extern crate rest_in_rust;
extern crate native_tls;
extern crate env_logger;

use rest_in_rust::*;
use native_tls::Pkcs12;


fn hello_world(_: &mut Request) -> Result<Response, HttpError> {
    Ok("hello encryption".into())
}

fn main() {
    let _ = env_logger::init();
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.get("/", hello_world);

    let  s = Server::new(addr,r);

    let der = include_bytes!("certificate.p12");
    let cert = Pkcs12::from_der(der, "password").unwrap();
    s.start_https(cert);
}
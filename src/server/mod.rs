use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use router::Router;
use std::net::SocketAddr;

pub struct Server {
    addr: SocketAddr,
    router: Option<Router>,
    protocol: Protocol,
}

enum Protocol {
    Http1,
    Https1()
}

impl Server {
    pub fn router(&mut self, r: Router) {
        self.router = Some(r);
    }

    pub fn http(addr: SocketAddr) -> Self {
        Server { addr: addr, router: None, protocol: Protocol::Http1 }
    }

    pub fn start(&mut self) {
        self.protocol.run();
    }
}

impl Default for Server {
    fn default() -> Self {
        Server { addr: "127.0.0.1:8080".parse().unwrap(), router: None, protocol: Protocol::Http1 }
    }
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = ::hyper::Error;
    type Future = ::futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, _req: Request) -> Self::Future {
        ::futures::future::ok(
            Response::new()
                .with_header(ContentLength("hello".len() as u64))
                .with_body("hello")
        )
    }
}

impl Protocol {
    fn run(&self) {}

    fn run_http() {}
    fn run_https() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_server() {
        use hyper::server::Http;
        let addr = "127.0.0.1:3000".parse().unwrap();
        let server = Http::new().bind(&addr, || Ok(Server::default())).unwrap();
        //        server.run().unwrap();
    }
}

use hyper::header::ContentLength;
use hyper::StatusCode;
use hyper::server::{Http, Request as HRequest, Response as HResponse, Service};
use router::{Router, Route};
use std::net::SocketAddr;
use request::Request;
use futures::future;

pub struct Server {
    addr: SocketAddr,
    router: Option<Router>,
    protocol: Protocol,
}

#[derive(Copy,Clone)]
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

    pub fn start(&self) {
        self.protocol.run(&self);
    }
}

impl Default for Server {
    fn default() -> Self {
        Server { addr: "127.0.0.1:8080".parse().unwrap(), router: None, protocol: Protocol::Http1 }
    }
}

impl Service for Server {
    type Request = HRequest;
    type Response = HResponse;
    type Error = ::hyper::Error;
    type Future = ::futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: HRequest) -> Self::Future {
        use route_recognizer::Params;
        future::ok({
            debug!("Got request {:?}",req);
            let handler: Option<(&Route, ::route_recognizer::Params)> = match self.router {
                Some(ref router) => router.resolve(req.method(), req.path()),
                None => None,
            };


            match handler {
                Some(tuple) => {
                    let mut request = Request::new(req, tuple.1);
                    let ref route = tuple.0;
                    debug!("Found route {}:{} with params {:?}", route.method, route.path, &request.params());
                    let ref r = route.callback;
                    let result = r.handle(&mut request);
                    match result {
                        Ok(response) => HResponse::from(response),
                        Err(e) => HResponse::from(e)
                    }

                }
                None => {
                    debug!("Found no route for {}:{}", req.method(), req.path());

                    HResponse::new()
                        .with_status(StatusCode::NotFound)
                        .with_body(format!("404, No resource found for {}", req.path()))
                }
            }
        })
    }
}

impl Protocol {
    fn run(&self, server: &Server) -> Result<(), ::hyper::Error> {
        match *self {
            Protocol::Http1 => self.run_http(server),
            Protocol::Https1() => unimplemented!(),
        }
    }

    fn run_http(&self, server: &Server) -> Result<(), ::hyper::Error> {
        //fixme return server, but what type does it have???
        use std::sync::Arc;
        let ref addr = server.addr;
        let val = Arc::new(server);
        let s = Http::new().bind(addr,  move || Ok(val.clone()))?;
        s.run();
        Ok(())
    }

    fn run_https() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_server() {
        use hyper::server::Http;
        let addr = "127.0.0.1:3000".parse().unwrap();
        let server = Http::new().bind(&addr,  move || Ok(Server::default())).unwrap();
        //        server.run().unwrap();
    }
}

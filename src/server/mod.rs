use hyper::header::ContentLength;
use hyper::server::{Http, Request as HRequest, Response as HResponse, Service};
use router::{Router,Route};
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
        self.protocol.run(&self.addr,&self);
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

        let ref handler: Option<(&Route, ::route_recognizer::Params)>  = match self.router {
            Some(ref router) => router.resolve(req.method(),req.path()),
            None => None,
        };

        match handler {
            Some((route,params)) => "".as_ref(),
            None => "".as_ref()
        }

        if handler.is_some() {
            let (route,params): (&Route,Params) = handler.unwrap();
        }


        ::futures::future::ok(

            HResponse::new()
                .with_header(ContentLength("hello".len() as u64))
                .with_body("hello")
        )
    }
}

impl Protocol {
    fn run(&self, addr: &SocketAddr, server: &Server)  -> Result<(),::hyper::Error> {
        match *self {
            Protocol::Http1 => self.run_http(addr,server),
            Protocol::Https1() => unimplemented!(),
        }
    }

    fn run_http(&self,addr: &SocketAddr, server: &Server) -> Result<(),::hyper::Error> {//fixme return server, but what type does it have???
        let server = Http::new().bind(&addr, || Ok(Server::default()))?;
        server.run()
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
        let server = Http::new().bind(&addr, || Ok(Server::default())).unwrap();
        //        server.run().unwrap();
    }
}

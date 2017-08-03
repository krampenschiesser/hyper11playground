use hyper::StatusCode;
use hyper::server::{Http, Request as HRequest, Response as HResponse, Service};
use router::{Router, Route};
use std::net::SocketAddr;
use request::Request;
use futures::future;
use std::sync::Arc;
use route_recognizer::Params;
use native_tls::Pkcs12;

pub struct Server {
    addr: SocketAddr,
    router: Arc<Router>,
}

struct InternalServer {
    router: Arc<Router>
}

//#[derive(Copy, Clone)]
enum Protocol {
    Http,
    Https(Pkcs12)
}

impl Server {
    pub fn new(addr: SocketAddr, r: Router) -> Self {
        Server { addr: addr, router: Arc::new(r) }
    }

    pub fn start_http(self) -> Result<(), ::hyper::Error> {
        Protocol::Http.run(self)
    }

    pub fn start_https(self, pkcs: Pkcs12) -> Result<(), ::hyper::Error> {
        Protocol::Https(pkcs).run(self)
    }
}

impl Default for Server {
    fn default() -> Self {
        Server { addr: "127.0.0.1:8080".parse().unwrap(), router: Arc::new(Router::new()) }
    }
}

impl Service for InternalServer {
    type Request = HRequest;
    type Response = HResponse;
    type Error = ::hyper::Error;
    type Future = ::futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: HRequest) -> Self::Future {
        debug!("Got request {:?}", req);
        future::ok({
            debug!("Resolving route for {:?}", req);
            let ref router: Arc<Router> = self.router;

            let r = router.clone();
            let result = r.resolve(req.method(), req.path());
            match result {
                Some(tuple) => {
                    let res = self.handle_route(req, tuple);
                    match res {
                        Ok(resp) => HResponse::from(resp),
                        Err(err) => HResponse::from(err)
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

impl InternalServer {
    fn handle_route(&self, req: HRequest, tuple: (&Route, Params)) -> Result<::response::Response, ::error::HttpError> {
        let mut request = Request::new(req, tuple.1);
        let ref route = tuple.0;
        debug!("Found route {}:{} with params {:?}", route.method, route.path, &request.params());
        let ref r = route.callback;
        r.handle(&mut request)
    }
}


impl Protocol {
    fn run(self, server: Server) -> Result<(), ::hyper::Error> {
        match self {
            Protocol::Http => Self::run_http(server),
            Protocol::Https(pkcs) => Self::run_https(pkcs, server),
        }
    }

    fn run_https(pkcs: Pkcs12, server: Server) -> Result<(), ::hyper::Error> {
        use futures::future::{ok, Future};
        use hyper::server::Http;
        use hyper::{Request, Response, StatusCode};
        use native_tls::{TlsAcceptor, Pkcs12};
        use tokio_proto::TcpServer;
        use tokio_service::Service;
        use tokio_tls::proto;

        let tls_cx = TlsAcceptor::builder(pkcs).unwrap()
            .build().unwrap();

        let proto = proto::Server::new(Http::new(), tls_cx);

        let addr = server.addr.clone();
        let srv = TcpServer::new(proto, addr);
        let router = server.router;
        srv.serve(move || Ok(InternalServer { router: router.clone() }));
        Ok(())
    }

    fn run_http(server: Server) -> Result<(), ::hyper::Error> {
        //fixme return server, but what type does it have???
        let addr = server.addr.clone();
        let router = server.router;
        let s = Http::new().bind(&addr, move || Ok(InternalServer { router: router.clone() }))?;
        s.run()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::header::ContentLength;

    struct TestServer;

    impl ::hyper::server::Service for TestServer {
        type Request = ::hyper::Request;
        type Response = ::hyper::Response;
        type Error = ::hyper::Error;
        type Future = ::futures::future::FutureResult<Self::Response, Self::Error>;

        fn call(&self, _req: Self::Request) -> Self::Future {
            ::futures::future::ok(
                Self::Response::new()
                    .with_header(ContentLength("test".len() as u64))
                    .with_body("test")
            )
        }
    }

    #[test]
    fn test_start_server() {
        use hyper::server::Http;
        //        let addr = "127.0.0.1:3000".parse().unwrap();
        let s = TestServer {};
        //        let server: ::hyper::Server<Server, ::hyper::Body> = Http::new().bind(&addr, move || Ok(s)).unwrap();
        //        server.run().unwrap();
    }
}

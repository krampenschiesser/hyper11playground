use hyper::StatusCode;
use hyper::server::{Request as HRequest, Response as HResponse, Service};
use router::{Router, Route};
use std::net::SocketAddr;
use request::Request;
use futures::future;
use std::sync::Arc;
use request::Params;
use native_tls::Pkcs12;
use state::Container;
use std::sync::atomic::{AtomicBool, Ordering};

mod protocol;

use self::protocol::Protocol;

pub struct Server {
    addr: SocketAddr,
    router: Arc<Router>,
    state: Arc<Container>,
}

struct InternalServer {
    state: Arc<Container>,
    router: Arc<Router>
}

pub struct ServerStopper {
    stop: AtomicBool,
}

impl ServerStopper {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

impl Default for ServerStopper {
    fn default() -> Self {
        ServerStopper { stop: AtomicBool::new(false) }
    }
}

impl ::futures::Future for ServerStopper {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        if self.stop.load(Ordering::SeqCst) {
            let ready = ::futures::Async::Ready(());
            Ok(ready)
        } else {
            let noready = ::futures::Async::NotReady;
            Ok(noready)
        }
    }
}

impl Server {
    pub fn new(addr: SocketAddr, r: Router) -> Self {
        Server { addr: addr, router: Arc::new(r), state: Arc::new(Container::new()) }
    }

    pub fn start_http(self) -> Result<ServerStopper, ::hyper::Error> {
        Protocol::Http.run(self)
    }
    pub fn add_state<T: Send + Sync + 'static>(&self, state: T) {
        if !self.state.set::<T>(state) {
            error!("State for this type is already being managed!");
            panic!("Aborting due to duplicately managed state.");
        }
    }

    pub fn start_https(self, pkcs: Pkcs12) -> Result<ServerStopper, ::hyper::Error> {
        Protocol::Https(pkcs).run(self)
    }
}

impl Default for Server {
    fn default() -> Self {
        Server { addr: "127.0.0.1:8080".parse().unwrap(), router: Arc::new(Router::new()), state: Arc::new(Container::new()) }
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
            let method = ::hyper_conversion::convert_method(req.method());
            let result = r.resolve(&method, req.path());
            match result {
                Some((route, params)) => {
                    let res = self.handle_route(req, route, params.into());
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
    fn handle_route(&self, req: HRequest, route: &Route, params: Params) -> Result<::response::Response, ::error::HttpError> {
        let mut request = Request::from_hyper(req, &self.state, params);
        debug!("Found route {}:{} with params {:?}", route.method, route.path, &request.params());
        let ref r = route.callback;
        r.handle(&mut request)
    }
}

#[cfg(test)]
mod tests {}

use std::io;
use http::Response;
use futures::future;
use tokio_service::Service;
use tokio_proto::TcpServer;
use ::request::{Request, Body};
use ::router::Router;
use state::Container;
use std::sync::atomic::{AtomicBool, Ordering};
use native_tls::Pkcs12;


use ::error::HttpError;
use std::net::SocketAddr;
use std::sync::Arc;

mod codec;

use self::codec::{Http, HttpCodecCfg, DecodingResult};

pub struct Server {
    addr: SocketAddr,
    router: Arc<Router>,
    state: Arc<Container>,
    stopper: ServerStopper,
}

struct InternalServer {
    state: Arc<Container>,
}

#[derive(Debug, Clone)]
pub struct ServerStopper {
    stop: Arc<AtomicBool>,
}

impl ServerStopper {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

impl Default for ServerStopper {
    fn default() -> Self {
        ServerStopper { stop: Arc::new(AtomicBool::new(false)) }
    }
}

impl Service for InternalServer {
    type Request = DecodingResult;
    type Response = Response<Body>;
    type Error = io::Error;
    type Future = future::Ok<Self::Response, io::Error>;

    fn call(&self, req: DecodingResult) -> Self::Future {
        let (req, handler, params) = match req {
            DecodingResult::BodyTooLarge => return future::ok(HttpError::bad_request("Request too large").into()),
            DecodingResult::HeaderTooLarge => return future::ok(HttpError::bad_request("Header too large").into()),
            DecodingResult::RouteNotFound => return future::ok(HttpError::not_found(Some("Route not found")).into()),
            DecodingResult::Ok(res) => res
        };
        debug!("Got request {:?}", req);

        let mut request = Request::new(req, &self.state, params);
        let res = handler.handle(&mut request);

        match res {
            Ok(resp) => {
                trace!("Successfully handled request. Response: {:?}", &resp);
                future::ok(resp.into_inner())
            },
            Err(err) => {
                warn!("Failed to handle {:?}", &err);
                future::ok(::response::Response::from(err).into_inner())
            }
        }
    }
}

impl Server {
    pub fn new(addr: SocketAddr, r: Router) -> Self {
        Server { stopper: ServerStopper::default(), addr: addr, router: Arc::new(r), state: Arc::new(Container::new()) }
    }

    pub fn start_http_non_blocking(self) -> Result<ServerStopper, ()> {
        use std::thread::spawn;
        let stopper = self.stopper.clone();

        spawn(|| self.start_http());
        Ok(stopper)
    }

    pub fn start_http(self) {
        //fixme currently shutdown not supported by tcpserver, next version  -> Result<ServerStopper, ()> {
        let addr = self.addr.clone();
        let router = self.router;
        let state = self.state;
        let stopper = self.stopper;
        state.set(stopper);
        let http = Http { router: router.clone(), config: HttpCodecCfg::default() };
        TcpServer::new(http, addr).serve(move || Ok(InternalServer { state: state.clone() }));

        //        let stopper = ServerStopper { stop: Arc::new(::std::sync::atomic::AtomicBool::new(false)) };
        //        Ok(stopper)
    }

    pub fn start_https(self, pkcs: Pkcs12) {
        //fixme currently shutdown not supported by tcpserver, next version  -> Result<ServerStopper, ()> {
        use native_tls::TlsAcceptor;
        use tokio_proto::TcpServer;
        use tokio_tls::proto;

        let tls_cx = TlsAcceptor::builder(pkcs).unwrap()
            .build().unwrap();

        let router = self.router;
        let http = Http { router: router.clone(), config: HttpCodecCfg::default() };
        let proto = proto::Server::new(http, tls_cx);

        let addr = self.addr.clone();
        let srv = TcpServer::new(proto, addr);
        let state = self.state;
        srv.serve(move || Ok(InternalServer { state: state.clone() }));

        //        Ok(ServerStopper::default())
    }

    pub fn add_state<T: Send + Sync + 'static>(&self, state: T) {
        if !self.state.set::<T>(state) {
            error!("State for this type is already being managed!");
            panic!("Aborting due to duplicately managed state.");
        }
    }


    pub fn get_stopper(&self) -> ServerStopper {
        self.stopper.clone()
    }
}

impl Default for Server {
    fn default() -> Self {
        Server { stopper: ServerStopper::default(), addr: "127.0.0.1:8080".parse().unwrap(), router: Arc::new(Router::new()), state: Arc::new(Container::new()) }
    }
}

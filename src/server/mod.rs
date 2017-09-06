// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::io;
use http::Response;
use futures::future;
use tokio_service::Service;
use tokio_proto::TcpServer;
use ::request::Request;
use ::body::Body;
use ::router::{Threading, Router, InternalRouter};
use state::Container;
use std::sync::atomic::{AtomicBool, Ordering};
use native_tls::Pkcs12;
use ::error::HttpError;
use std::net::SocketAddr;
use std::sync::Arc;
use futures_cpupool::{CpuPool, Builder as PoolBuilder};

mod codec;
pub mod tester;

use self::codec::{Http, HttpCodecCfg, DecodingResult, DecodedRequest};

pub struct Server {
    pool: CpuPool,
    addr: SocketAddr,
    router: Arc<InternalRouter>,
    state: Arc<Container>,
    stopper: ServerStopper,
    codec_cfg: HttpCodecCfg,
}

struct InternalServer {
    pool: CpuPool,
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
        let dec_req = match req {
            DecodingResult::BodyTooLarge => return future::ok(HttpError::bad_request("Request too large").into()),
            DecodingResult::HeaderTooLarge => return future::ok(HttpError::bad_request("Header too large").into()),
            DecodingResult::RouteNotFound => return future::ok(HttpError::not_found(Some("Route not found")).into()),
            DecodingResult::Ok(res) => res
        };

        //        self.pool.spawn()

        let DecodedRequest { request: req, route, params } = dec_req;
        debug!("Got request {:?}", req);

        let r = || {
            let mut request = Request::new(req, &self.state, params);
            let res = route.callback.handle(&mut request);

            match res {
                Ok(resp) => {
                    trace!("Successfully handled request. Response: {:?}", &resp);
                    Ok(resp.into_inner())
                }
                Err(err) => {
                    warn!("Failed to handle {:?}", &err);
                    Ok(::response::Response::from(err).into_inner())
                }
            }
        };
        match route.threading {
            Threading::SEPERATE => self.pool.spawn_fn(r),
            Threading::SAME => future::ok(r())
        }
    }
}

impl Server {
    pub fn new(addr: SocketAddr, r: Router) -> Self {
        let internal_router = InternalRouter::new(r);
        let pool = PoolBuilder::new().name_prefix("RIR_Worker").pool_size(20).create();
        Server { codec_cfg: HttpCodecCfg::default(), stopper: ServerStopper::default(), addr: addr, router: Arc::new(internal_router), state: Arc::new(Container::new()), pool }
    }

    pub fn set_codec_cfg(&mut self, cfg: HttpCodecCfg) {
        self.codec_cfg = cfg;
    }

    pub fn set_thread_pool_size(&mut self, size: usize) {
        self.pool = PoolBuilder::new().name_prefix("RIR_Worker").pool_size(size).create();
    }

    pub fn start_http_non_blocking(self) -> Result<ServerStopper, ()> {
        use std::thread::spawn;
        let stopper = self.stopper.clone();

        spawn(|| self.start_http());
        Ok(stopper)
    }

    pub fn start_testing(self) -> self::tester::ServerTester {
        use self::tester::ServerTester;

        ServerTester::new(self.router, self.state)
    }

    pub fn start_http(self) {
        //fixme currently shutdown not supported by tcpserver, next version  -> Result<ServerStopper, ()> {
        let addr = self.addr.clone();
        let state = self.state;
        state.set(self.stopper);
        let http = Http { router: self.router.clone(), config: self.codec_cfg };
        let pool = self.pool;
        TcpServer::new(http, addr).serve(move || Ok(InternalServer { state: state.clone(), pool: pool.clone() }));

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
        let http = Http { router: router.clone(), config: self.codec_cfg };
        let proto = proto::Server::new(http, tls_cx);

        let addr = self.addr.clone();
        let srv = TcpServer::new(proto, addr);
        let state = self.state;
        let pool = self.pool;
        srv.serve(move || Ok(InternalServer { state: state.clone(), pool: pool.clone() }));

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
        Server {
            pool: PoolBuilder::new().name_prefix("RIR_Worker").pool_size(20).create(),
            codec_cfg: HttpCodecCfg::default(),
            stopper: ServerStopper::default(),
            addr: "127.0.0.1:8080".parse().unwrap(),
            router: Arc::new(InternalRouter::new(Router::new())),
            state: Arc::new(Container::new())
        }
    }
}

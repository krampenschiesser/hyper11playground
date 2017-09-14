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
use mime_sniffer::MimeTypeSniffer;

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

use futures::Future;

impl Service for InternalServer {
    type Request = DecodingResult;
    type Response = Response<Body>;
    type Error = io::Error;
    type Future = Box<Future<Item=Response<Body>, Error=io::Error>>;

    fn call(&self, req: DecodingResult) -> Self::Future {
        let dec_req = match req {
            DecodingResult::BodyTooLarge => return Box::new(future::ok(HttpError::bad_request("Request too large").into())),
            DecodingResult::HeaderTooLarge => return Box::new(future::ok(HttpError::bad_request("Header too large").into())),
            DecodingResult::RouteNotFound => return Box::new(future::ok(HttpError::not_found("Route not found").into())),
            DecodingResult::Ok(res) => res
        };

        let DecodedRequest { request: req, route, params } = dec_req;
        debug!("Got request {:?}", req);
        let state = self.state.clone();
        let local_route = route.clone();

        let r = move || {
            let mut request = Request::new(req, state, params);
            let res = local_route.callback.handle(&mut request);
            let res = if let Ok(res) = res {
                Ok(enhance_content_type(res))
            } else {
                res
            };

            match res {
                Ok(resp) => {
                    trace!("Successfully handled request. Response: {:?}", &resp);
                    future::ok(resp.into_inner())
                }
                Err(err) => {
                    warn!("Failed to handle {:?}", &err);
                    future::ok(::response::Response::from(err).into_inner())
                }
            }
        };

        match route.threading {
            Threading::SAME => Box::new(r()),
            Threading::SEPERATE => Box::new(self.pool.spawn_fn(r)),
        }
    }
}

fn enhance_content_type(response: ::response::Response) -> ::response::Response {
    let mut resp = response.into_inner();
    let key = ::http::header::CONTENT_TYPE;
    let no_content_type = {
        resp.headers().get(&key).is_none()
    };
    if no_content_type {
        let hv: Option<::http::header::HeaderValue> = match resp.body().inner() {
            &Some(ref vec) => {
                let mime_type = vec.sniff_mime_type();
                debug!("Found mime type {:?} for {:?}",mime_type, resp.body());
                match mime_type {
                    Some(mime_type) => {
                        let r = ::http::header::HeaderValue::from_str(mime_type);
                        r.ok()
                    }
                    None => None,
                }
            }
            &None => None,
        };
        if let Some(hv) = hv {
            resp.headers_mut().insert(key, hv);
        }
    }
    ::response::Response::from_http(resp)
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_content_type() {
        test_content_type(None, b"<body");
        test_content_type(None, b"<note><to>Tove</to></note>s");
        test_content_type(None, b"@font-face{font-family:Work ");
    }

    fn test_content_type(content_type: Option<&str>, body: &[u8]) {
        let resp = ::response::Response::from(body);

        let resp = enhance_content_type(resp);
        let o = resp.headers().get(::http::header::CONTENT_TYPE);

        match content_type {
            Some(content_type) => {
                let h = ::http::header::HeaderValue::from_str(content_type).unwrap();
                assert_eq!(Some(&h), o)
            }
            None => {
                assert_eq!(None, o)
            }
        }
    }
}

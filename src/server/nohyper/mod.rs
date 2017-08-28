use std::io;
use http::Response;

use futures::future;
use tokio_service::Service;

mod codec;

use self::codec::{HttpCodec, HttpCodecCfg, DecodingResult};

struct Server;

impl Service for Server {
    type Request = DecodingResult;
    type Response = Response<Option<Vec<u8>>>;
    type Error = io::Error;
    type Future = future::Ok<Self::Response, io::Error>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let response = Response::new(Some("hello_world".into()));
        future::ok(response)
    }
}


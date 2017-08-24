use std::io;
use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;
use http::{Request, Response, Method, Uri, Version};
use http::header::{HeaderValue, HeaderName, HeaderMap};
use http::request::Builder as RequestBuilder;
use std::str::FromStr;
use ::router::Router;
use std::sync::Arc;

use futures::future;
use tokio_proto::TcpServer;
use tokio_service::Service;

mod codec;

use self::codec::{HttpCodec,HttpCodecCfg,DecodingResult};

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


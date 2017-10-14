// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::io;
use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;
use http::{Request, Response, Method, Uri, Version};
use http::header::{HeaderValue, HeaderName, HeaderMap};
use http::request::Builder as RequestBuilder;
use std::str::FromStr;
use ::router::{InternalRouter, Route};
use std::sync::Arc;

use ::body::Body;
use ::request::Params;

pub struct Http {
    pub router: Arc<InternalRouter>,
    pub config: HttpCodecCfg,
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for Http {
    type Request = DecodingResult;
    type Response = Response<Body>;
    type Transport = Framed<T, HttpCodec>;
    type BindTransport = io::Result<Framed<T, HttpCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, HttpCodec>> {
        let codec = HttpCodec { config: self.config.clone(), router: self.router.clone(), request: None };
        Ok(io.framed(codec))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct HttpCodecCfg {
    max_reuest_header_len: usize,
    max_body_size: usize,
    max_headers: usize,
}

impl Default for HttpCodecCfg {
    fn default() -> Self {
        HttpCodecCfg { max_reuest_header_len: 8000, max_body_size: 20_000_000, max_headers: 64 }
    }
}

pub struct HttpCodec {
    config: HttpCodecCfg,
    router: Arc<InternalRouter>,
    request: Option<PartialResultWithBody>,
}

struct PartialResultWithBody {
    body_length: BodyLength,
    request: Request<Body>,
    handler: Arc<Route>,
    params: Params,
}

impl PartialResultWithBody {
    fn append_buf(&mut self, buf: &mut BytesMut) -> bool {
        if let &mut Some(ref mut body) = self.request.body_mut().inner_mut() {
            let buf_len = buf.len();
            let current_length = body.len();
            let remaining_length = self.body_length - current_length;

            if buf_len > remaining_length {
                let buf_body = buf.split_to(self.body_length);
                body.extend_from_slice(buf_body.as_ref());
                true
            } else {
                body.extend_from_slice(buf.as_ref());
                buf.clear();
                buf_len == remaining_length
            }
        } else {
            return false;
        }
    }
}

type BodyLength = usize;

impl Default for HttpCodec {
    fn default() -> Self {
        HttpCodec { config: HttpCodecCfg::default(), router: Arc::new(InternalRouter::new(::router::Router::new())), request: None }
    }
}

pub enum DecodingResult {
    RouteNotFound,
    HeaderTooLarge,
    BodyTooLarge,
    Ok(DecodedRequest),
}

pub struct DecodedRequest {
    pub request: Request<Body>,
    pub route: Arc<Route>,
    pub params: Params,
}

impl ::std::fmt::Debug for DecodingResult {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::DecodingResult::*;

        match *self {
            RouteNotFound => write!(f, "RouteNotFound"),
            HeaderTooLarge => write!(f, "HeaderTooLarge"),
            BodyTooLarge => write!(f, "BodyTooLarge"),
            Ok(ref res) => write!(f, "Ok({:?} [{:?}])", res.request, res.params),
        }
    }
}

impl Decoder for HttpCodec {
    type Item = DecodingResult;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<DecodingResult>> {
        let completed_body = match self.request {
            Some(ref mut partial) => {
                trace!("Got existing partial body, appending");
                partial.append_buf(buf)
            }
            None => false
        };
        if completed_body {
            let o = self.request.take();
            if let Some(partial) = o {
                trace!("Completed partial body, returning");
                let PartialResultWithBody { body_length: _, request, handler: route, params } = partial;
                let dec_req = DecodedRequest { request, params, route };
                let decoding_result = DecodingResult::Ok(dec_req);
                return Ok(Some(decoding_result));
            }
        }

        let result = parse(self, buf)?;
        //fixme remove body handling for non valid requests
        if let None = result {
            if buf.len() > self.config.max_reuest_header_len {
                buf.clear();
                trace!("Header exceeds limit, will return error");
                return Ok(Some(DecodingResult::HeaderTooLarge));
            } else {
                trace!("Not enough data for header");
                return Ok(None);
            }
        }
        let (method, uri, version, header_map, body_complete, body_start, body_length) = result.unwrap();
        buf.split_to(body_start);//remove part of buffer

        if body_length > self.config.max_body_size {
            buf.clear();
            trace!("Body exceeds limit, will return error");
            return Ok(Some(DecodingResult::BodyTooLarge));
        }

        let o = self.router.resolve(&method, uri.path());
        if let Some((route, params)) = o {
            trace!("Found route for {} {}", &method, uri.path());
            let mut b = RequestBuilder::new();
            b.method(method);
            b.uri(uri);
            b.version(version);

            let mut request = if body_length > 0 {
                let max_init_content = 4000;
                let capacity = ::std::cmp::min(body_length, max_init_content);
                let vec = Vec::with_capacity(capacity);
                b.body(Body(Some(vec))).map_err(|e| io_error(e))?
            } else {
                b.body(Body(None)).map_err(|e| io_error(e))?
            };
            *request.headers_mut() = header_map;

            if body_complete {
                let body = get_body(buf, 0, body_length);
                *request.body_mut() = body;
                debug!("Got Request: {:?}", request);
                let dec_req = DecodedRequest { request, params: params.into(), route };
                let decoding_result = DecodingResult::Ok(dec_req);
                Ok(Some(decoding_result))
            } else {
                debug!("Got Request with incomplete body: {:?}", request);
                trace!("Body not complete. Got {} of {} total bytes", buf.len(), body_length);
                self.request = Some(PartialResultWithBody { request, params: params.into(), handler: route, body_length });
                Ok(None)
            }
        } else {
            buf.clear();
            Ok(Some(DecodingResult::RouteNotFound))
        }
    }
}

fn get_body(buf: &mut BytesMut, content_start: usize, content_length: usize) -> Body {
    if content_length > 0 {
        println!("Contentlength={}, contentstart={}, buf.len={}", content_length, content_start, buf.len());
        let split = buf.split_off(content_start);
        let v: Vec<u8> = Vec::from(split.as_ref());
        Body(Some(v))
    } else {
        Body(None)
    }
}

fn parse(codec: &mut HttpCodec, buf: &mut BytesMut) -> Result<Option<(Method, Uri, Version, HeaderMap<HeaderValue>, bool, usize, usize)>, ::std::io::Error> {
    use httparse;

    let mut headers = vec![::httparse::EMPTY_HEADER; codec.config.max_headers];//fixme don't allocate, immediately but grow on demand by handling parse error
    let mut r = httparse::Request::new(headers.as_mut());
    let status = r.parse(buf.as_ref()).map_err(|e| {
        let msg = format!("failed to parse http request: {:?}", e);
        ::std::io::Error::new(io::ErrorKind::Other, msg)
    })?;
    let amt = match status {
        httparse::Status::Complete(amt) => amt,
        httparse::Status::Partial => return Ok(None)
    };

    let content_length = get_content_length(&r);
    let total_length = amt + content_length;

    let method = parse_method(&r).map_err(|e| io_error(e))?;
    let uri = parse_uri(&r).map_err(|e| io_error(e))?;
    let version = parse_version(&r);
    let headers = translate_headers(&r)?;

    Ok(Some((method, uri, version, headers, buf.len() >= total_length, amt, content_length)))
}

fn io_error<T: ::std::fmt::Debug>(t: T) -> ::std::io::Error {
    ::std::io::Error::new(::std::io::ErrorKind::Other, format!("{:?}", t))
}


fn parse_method(req: &::httparse::Request) -> Result<Method, ::http::method::InvalidMethod> {
    Method::from_bytes(req.method.unwrap().as_ref())
}

fn parse_uri(req: &::httparse::Request) -> Result<Uri, ::http::uri::InvalidUri> {
    Uri::from_str(req.path.unwrap())
}

fn parse_version(req: &::httparse::Request) -> Version {
    match req.version.unwrap() {
        2 => ::http::Version::HTTP_2,
        1 => ::http::Version::HTTP_11,
        0 => ::http::Version::HTTP_10,
        _ => ::http::Version::HTTP_11
    }
}

fn get_content_length(req: &::httparse::Request) -> usize {
    use ::http::header::CONTENT_LENGTH;

    if let Some(header) = req.headers.iter().filter(|h| h.name == CONTENT_LENGTH).next() {
        let amount_str = ::std::str::from_utf8(header.value).unwrap_or("");
        let value = usize::from_str(amount_str).unwrap_or(0);
        trace!("Got content-length={}", value);
        value
    } else {
        0
    }
}

fn translate_headers(req: &::httparse::Request) -> Result<::http::header::HeaderMap<::http::header::HeaderValue>, ::std::io::Error> {
    let mut map = HeaderMap::new();
    for header in req.headers.iter() {
        let name = header.name;
        let value = header.value;

        let header_name = HeaderName::from_str(name).map_err(|e| io_error(e))?;
        let header_value = HeaderValue::from_bytes(value).map_err(|e| io_error(e))?;

        map.insert(header_name, header_value);
    }

    Ok(map)
}


impl Encoder for HttpCodec {
    type Item = Response<Body>;
    type Error = io::Error;

    fn encode(&mut self, msg: Response<Body>, buf: &mut BytesMut) -> io::Result<()> {
        let status_line = format!("{:?} {} {}\r\n", msg.version(), msg.status().as_u16(), msg.status());
        buf.extend_from_slice(status_line.as_bytes());
        for (key, value) in msg.headers().iter() {
            let val: &[u8] = key.as_ref();
            buf.extend_from_slice(val);
            buf.extend_from_slice(b": ");
            let val: &[u8] = value.as_ref();
            buf.extend_from_slice(val);
            buf.extend_from_slice(b"\r\n");
        }

        let length_header = msg.headers().iter().find(|h| h.0 == ::http::header::CONTENT_LENGTH);

        if let &Some(ref vec) = msg.body().inner() {
            if length_header.is_none() {
                buf.extend_from_slice(b"Content-Length: ");
                buf.extend_from_slice(format!("{}", vec.len()).as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
        }
        buf.extend_from_slice(b"\r\n");
        if let &Some(ref vec) = msg.body().inner() {
            buf.extend_from_slice(vec.as_slice());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    extern crate env_logger;

    use router::{Router, InternalRouter};

    const RAW_GET: &'static [u8] = b"GET / HTTP/1.1\r\n";
    const RAW_HEADER: &'static [u8] = b"Host: Nirvana\r\nConnection: keep-alive\r\n";

    fn handle(_: &mut ::request::Request) -> Result<::response::Response, ::error::HttpError> {
        Ok("".into())
    }

    #[test]
    fn test_header_too_long_incomplete() {
        let mut bytes = BytesMut::from(RAW_GET.as_ref());
        bytes.extend_from_slice(RAW_HEADER.as_ref());

        let config = HttpCodecCfg { max_reuest_header_len: 30, max_body_size: 10, max_headers: 10 };
        let r = parse(bytes, config);

        match r {
            DecodingResult::HeaderTooLarge => return,
            r => panic!("wrong return value {:?}", r)
        }
    }

    #[test]
    fn test_body_too_long() {
        let mut bytes = BytesMut::from(RAW_GET.as_ref());
        bytes.extend_from_slice(RAW_HEADER.as_ref());
        bytes.extend_from_slice(b"Content-Length: 30\r\n\r\n".as_ref());

        let config = HttpCodecCfg { max_reuest_header_len: 8000, max_body_size: 10, max_headers: 10 };
        let r = parse(bytes, config);

        match r {
            DecodingResult::BodyTooLarge => return,
            r => panic!("wrong return value {:?}", r)
        }
    }

    fn parse(mut bytes: BytesMut, config: HttpCodecCfg) -> DecodingResult {
        let router = Arc::new(InternalRouter::new(Router::new()));
        let mut codec = HttpCodec { config, router, request: None };
        let r = codec.decode(&mut bytes);
        match r {
            Ok(s) => match s {
                Some(s) => s,
                None => panic!("Decoder not ready and waiting -> but should abort"),
            },
            Err(e) => panic!("Got error from decoder {:?}", e),
        }
    }

    #[test]
    fn test_body_missing() {
        let mut bytes = BytesMut::from(RAW_GET.as_ref());
        bytes.extend_from_slice(RAW_HEADER.as_ref());
        bytes.extend_from_slice(b"Content-Length: 30\r\n\r\n".as_ref());

        let mut r = Router::new();
        r.get("/", handle);
        let cfg = HttpCodecCfg::default();
        let mut codec = HttpCodec { config: cfg, router: Arc::new(InternalRouter::new(r)), request: None };
        let r = codec.decode(&mut bytes);
        assert_that(&r).is_ok();
        assert_that(&r.unwrap()).is_none();
    }

    #[test]
    fn test_route_not_found() {
        let mut bytes = BytesMut::from(RAW_GET.as_ref());
        bytes.extend_from_slice(RAW_HEADER.as_ref());
        bytes.extend_from_slice(b"Content-Length: 30\r\n\r\n".as_ref());

        let mut codec = HttpCodec::default();
        let r = codec.decode(&mut bytes);
        assert_that(&r).is_ok();
        let r = r.unwrap();
        assert_that(&r).is_some();
        let r = r.unwrap();

        match r {
            DecodingResult::RouteNotFound => return,
            r => panic!("wrong return value {:?}", r)
        }
    }

    #[test]
    fn test_wait_for_header() {
        let mut bytes = BytesMut::from(RAW_GET.as_ref());
        bytes.extend_from_slice(RAW_HEADER.as_ref());

        let mut codec = HttpCodec::default();
        let r = codec.decode(&mut bytes);
        assert_that(&r).is_ok();
        assert_that(&r.unwrap()).is_none();
    }

    #[test]
    fn receive_body_two_parts() {
        let mut r = Router::new();
        r.get("/", handle);

        let mut codec = HttpCodec { config: HttpCodecCfg::default(), router: Arc::new(InternalRouter::new(r)), request: None };

        let mut bytes = BytesMut::from(RAW_GET.as_ref());
        bytes.extend_from_slice(RAW_HEADER.as_ref());
        bytes.extend_from_slice(b"Content-Length: 11\r\n".as_ref());
        bytes.extend_from_slice(b"\r\n".as_ref());
        bytes.extend_from_slice(b"Hello ");

        {
            let r = codec.decode(&mut bytes);
            assert_that(&r).is_ok();
            assert_that(&r.unwrap()).is_none();
        }
        assert_eq!(6, bytes.len());
        assert!(&codec.request.is_some());

        bytes.extend_from_slice(b"World");
        {
            let r = codec.decode(&mut bytes);
            assert_that(&r).is_ok();
            let o = r.unwrap();
            assert_that(&o).is_some();
            match o.unwrap() {
                DecodingResult::Ok(res) => {
                    let (_, body) = res.request.into_parts();
                    let b = body.into_inner().unwrap();
                    let body_string = String::from_utf8(b).unwrap();
                    assert_eq!("Hello World", body_string);
                }
                _ => panic!("Got no result"),
            }
        }
    }

    #[test]
    fn post_simple() {
        let _ = env_logger::init();
        let mut r = Router::new();
        r.get("/", handle);

        let mut codec = HttpCodec { config: HttpCodecCfg::default(), router: Arc::new(InternalRouter::new(r)), request: None };

        let mut bytes = BytesMut::from(RAW_GET.as_ref());
        bytes.extend_from_slice(RAW_HEADER.as_ref());
        bytes.extend_from_slice(b"Content-Length: 4\r\n".as_ref());
        bytes.extend_from_slice(b"\r\n".as_ref());
        bytes.extend_from_slice(b"Hello");

        let r = codec.decode(&mut bytes);
        assert_that(&r).is_ok();
        let o = r.unwrap();
        assert_that(&o).is_some();
        match o.unwrap() {
            DecodingResult::Ok(res) => {
                let (_, body) = res.request.into_parts();
                let b = body.into_inner().unwrap();
                let body_string = String::from_utf8(b).unwrap();
                assert_eq!("Hello", body_string);
            }
            _ => panic!("Got no result"),
        }
    }
}

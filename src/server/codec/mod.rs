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
use ::handler::Handler;
use std::sync::Arc;

use ::request::Body;

pub struct Http {
    pub router: Arc<Router>,
    pub config: HttpCodecCfg,
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for Http {
    type Request = DecodingResult;
    type Response = Response<Body>;
    type Transport = Framed<T, HttpCodec>;
    type BindTransport = io::Result<Framed<T, HttpCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, HttpCodec>> {
        let codec = HttpCodec { config: self.config.clone(), router: self.router.clone() };
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
    router: Arc<Router>,
}

impl Default for HttpCodec {
    fn default() -> Self {
        HttpCodec { config: HttpCodecCfg::default(), router: Arc::new(Router::new()) }
    }
}

pub enum DecodingResult {
    RouteNotFound,
    HeaderTooLarge,
    BodyTooLarge,
    Ok((Request<Body>, Arc<Box<Handler>>, ::request::Params)),
}


impl ::std::fmt::Debug for DecodingResult {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::DecodingResult::*;

        match *self {
            RouteNotFound => write!(f, "RouteNotFound"),
            HeaderTooLarge => write!(f, "HeaderTooLarge"),
            BodyTooLarge => write!(f, "BodyTooLarge"),
            Ok(_) => write!(f, "Ok(...)"),
        }
    }
}

impl Decoder for HttpCodec {
    type Item = DecodingResult;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<DecodingResult>> {
        let result = parse(self, buf)?;
        if let None = result {
            if buf.len() > self.config.max_reuest_header_len {
                buf.clear();
                return Ok(Some(DecodingResult::HeaderTooLarge));
            } else {
                return Ok(None);
            }
        }
        let (method, uri, version, header_map, body_complete, content_start, content_length) = result.unwrap();
        if content_length > self.config.max_body_size {
            buf.clear();
            return Ok(Some(DecodingResult::BodyTooLarge));
        }

        let o = self.router.resolve(&method, uri.path());
        if let Some((route, params)) = o {
            if body_complete {
                let body = get_body(buf, content_start, content_length);
                let mut b = RequestBuilder::new();
                b.method(method);
                b.uri(uri);
                b.version(version);
                let mut request = b.body(body).map_err(|e| io_error(e))?;
                *request.headers_mut() = header_map;
                let decoding_result = DecodingResult::Ok((request, route.callback.clone(), params.into()));
                Ok(Some(decoding_result))
            } else {
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
        let split = buf.split_off(content_start);
        let v: Vec<u8> = Vec::from(split.as_ref());
        Some(v)
    } else {
        None
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

    let toslice = |a: &[u8]| {
        let start = a.as_ptr() as usize - buf.as_ptr() as usize;
        assert!(start < buf.len());
        (start, start + a.len())
    };

    let content_length = get_content_length(&r);
    let total_length = amt + content_length;

    let method = parse_method(&r).map_err(|e| io_error(e))?;
    let uri = parse_uri(&r).map_err(|e| io_error(e))?;
    let version = parse_version(&r);
    let headers = translate_headers(&r)?;

    Ok(Some((method, uri, version, headers, buf.len() == total_length, amt, content_length)))
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
        2 => ::http::version::HTTP_2,
        1 => ::http::version::HTTP_11,
        0 => ::http::version::HTTP_10,
        _ => ::http::version::HTTP_11
    }
}

fn get_content_length(req: &::httparse::Request) -> usize {
    if let Some(header) = req.headers.iter().filter(|h| h.name.as_bytes() == b"Content-Length").next() {
        let amount_str = ::std::str::from_utf8(header.value).unwrap_or("");
        let value = usize::from_str(amount_str).unwrap_or(0);
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
        //        response::encode(msg, buf);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    const RAW_GET: &'static [u8] = b"GET / HTTP/1.0\r\n";
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
        let router = Arc::new(Router::new());
        let mut codec = HttpCodec { config, router };
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
        let mut codec = HttpCodec { config: cfg, router: Arc::new(r) };
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
}
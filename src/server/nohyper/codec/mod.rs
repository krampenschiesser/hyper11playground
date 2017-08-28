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

pub struct Http {
    router: Arc<Router>,
    config: HttpCodecCfg,
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for Http {
    type Request = DecodingResult;
    type Response = Response<Option<Vec<u8>>>;
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
    //    max_headers: u16,
}

impl Default for HttpCodecCfg {
    fn default() -> Self {
        HttpCodecCfg { max_reuest_header_len: 8000, max_body_size: 20_000_000 }//, max_headers: 32 }
    }
}

pub struct HttpCodec {
    config: HttpCodecCfg,
    router: Arc<Router>,
}

pub enum DecodingResult {
    RouteNotFound,
    HeaderTooLarge,
    BodyTooLarge,
    Ok((Request<Option<Vec<u8>>>, Arc<Box<Handler>>)),
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
            }
            return Ok(None);
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
                let decoding_result = DecodingResult::Ok((request, route.callback.clone()));
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

fn get_body(buf: &mut BytesMut, content_start: usize, content_length: usize) -> Option<Vec<u8>> {
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
    use httparse::Header;

    let mut headers: Vec<Header> = Vec::with_capacity(32);
    let header_ref = headers.as_mut();
    let mut r = httparse::Request::new(header_ref);
    let status = r.parse(buf.as_ref()).map_err(|e| {
        let msg = format!("failed to parse http request: {:?}", e);
        ::std::io::Error::new(io::ErrorKind::Other, msg)
    })?;
    let amt = match status {
        httparse::Status::Complete(amt) => amt,
        httparse::Status::Partial => return Ok(None),
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
    type Item = Response<Option<Vec<u8>>>;
    type Error = io::Error;

    fn encode(&mut self, msg: Response<Option<Vec<u8>>>, buf: &mut BytesMut) -> io::Result<()> {
        //        response::encode(msg, buf);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_header_too_long() {}

    fn test_body_too_long() {}

    fn test_body_missing() {}

    fn test_route_not_found() {}

    fn test_parsing_errors() {}

    fn test_wait_for_body() {}

    fn test_wait_for_header() {}
}

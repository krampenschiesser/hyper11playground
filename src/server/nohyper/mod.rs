use std::io;
use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;
use http::{Request, Response, Method, Uri, Version};
use http::request::Builder as RequestBuilder;
use std::str::FromStr;
use std::error::Error;
use ::router::Router;
use std::sync::Arc;

pub struct Http;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for Http {
    type Request = Request<Option<Vec<u8>>>;
    type Response = Response<Option<Vec<u8>>>;
    type Transport = Framed<T, HttpCodec>;
    type BindTransport = io::Result<Framed<T, HttpCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, HttpCodec>> {
        Ok(io.framed(HttpCodec { config: HttpCodecCfg::default(), router: Arc::new(Router::new()) }))
    }
}

pub struct HttpCodecCfg {
    max_reuest_header_len: u16,
    max_body_size: u16,
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

impl HttpCodec {
    fn parse(&mut self, buf: &mut BytesMut) -> Result<Option<(RequestBuilder, bool)>, ::std::io::Error> {
        use httparse;
        use httparse::Header;

        let mut headers: Vec<Header> = Vec::with_capacity(32);
        let mut header_ref = headers.as_mut();
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

        let mut b = RequestBuilder::new();
        b.method(parse_method(&r).map_err(|e| io_error(e))?);
        b.uri(parse_uri(&r).map_err(|e| io_error(e))?);
        b.version(parse_version(&r));
        set_headers(&r, &mut b);

        Ok(Some((b, buf.len() == total_length)))
    }
}

impl Decoder for HttpCodec {
    type Item = Request<Option<Vec<u8>>>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Request<Option<Vec<u8>>>>> {
        use httparse;
        use httparse::Header;

        let result = HttpCodec::parse(self, buf)?;
        if let None = result {
            return Ok(None);
        }
        let (builder, body_complete) = result.unwrap();


        //       c let data = buf.split_to(total_length);


        //            Request::builder()
        //                .
        //        (toslice(r.method.unwrap().as_bytes()),
        //         toslice(r.path.unwrap().as_bytes()),
        //         r.version.unwrap(),
        //         r.headers
        //             .iter()
        //             .map(|h| (toslice(h.name.as_bytes()), toslice(h.value)))
        //             .collect(),
        //         amt)
        Err(::std::io::Error::new(io::ErrorKind::Other, "test"))
        //        Ok(Request {
        //            method: method,
        //            path: path,
        //            version: version,
        //            headers: headers,
        //            data: buf.split_to(amt),
        //        }.into())
    }
}

fn io_error<T: ::std::fmt::Debug>(t: T) -> ::std::io::Error {
    ::std::io::Error::new(::std::io::ErrorKind::Other, format!("{:?}", t))
}

impl Encoder for HttpCodec {
    type Item = Response<Option<Vec<u8>>>;
    type Error = io::Error;

    fn encode(&mut self, msg: Response<Option<Vec<u8>>>, buf: &mut BytesMut) -> io::Result<()> {
        //        response::encode(msg, buf);
        Ok(())
    }
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

fn set_headers(req: &::httparse::Request, b: &mut ::http::request::Builder) -> Result<(), ::std::io::Error> {
    use ::http::header::{HeaderName, HeaderValue};

    for header in req.headers.iter() {
        let name = header.name;
        let value = header.value;

        let header_name = HeaderName::from_str(name).map_err(|e| io_error(e))?;
        let header_value = HeaderValue::from_bytes(value).map_err(|e| io_error(e))?;

        b.header(header_name, header_value);
    }

    Ok(())
}
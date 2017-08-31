use http::Response as HttpResponse;
use std::ops::Deref;
use http::{StatusCode, HeaderMap, status};
use http::header::{HeaderValue, HeaderName};
use ::body::Body;

#[derive(Debug)]
pub struct Response {
    inner: HttpResponse<Body>,
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
    }

    pub fn moved_permanent<'a, T: AsRef<&'a str>>(url: T) -> Result<Response, ::error::HttpError> {
        let value: HeaderValue = HeaderValue::from_str(url.as_ref())?;
        Response::builder()
            .status(::http::status::MOVED_PERMANENTLY)
            .header(::http::header::LOCATION, value)
            .build()
    }

    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        self.inner.headers()
    }

    pub fn body(&self) -> &Body {
        self.inner.body()
    }

    pub fn into_inner(self) -> HttpResponse<Body> {
        self.inner
    }
}

pub struct ResponseBuilder {
    status: StatusCode,
    body: Body,
    header: HeaderMap<HeaderValue>,
}

impl ResponseBuilder {
    pub fn status<T: Into<StatusCode>>(mut self, status: T) -> Self {
        self.status = status.into();
        self
    }

    pub fn header_map(mut self, map: HeaderMap) -> Self {
        self.header = map;
        self
    }

    pub fn header<N: Into<HeaderName>, K: Into<HeaderValue>>(mut self, name: N, value: K) -> Self {
        self.header.insert(name.into(), value.into());
        self
    }
    pub fn header_str_value<'a, N: Into<HeaderName>, K: AsRef<&'a str>>(mut self, name: N, value: K) -> Result<Self, ::error::HttpError> {
        let value = HeaderValue::from_str(value.as_ref())?;
        self.header.insert(name.into(), value);
        Ok(self)
    }
    pub fn body<T: Into<Vec<u8>>>(mut self, body: T) -> Self {
        self.body = Body(Some(body.into()));
        self
    }

    pub fn build(self) -> Result<Response, ::error::HttpError> {
        let mut builder = HttpResponse::builder();
        builder.status(self.status);
        let mut inner = builder.body(self.body)?;
        *inner.headers_mut() = self.header;
        Ok(Response { inner })
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        ResponseBuilder { status: ::http::status::OK, body: Body(None), header: HeaderMap::new() }
    }
}

impl Deref for Response {
    type Target = HttpResponse<Body>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<String> for Response {
    fn from(val: String) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(status::OK);
        let x = builder.body(Body(Some(val.into()))).unwrap(); // in the code only Ok is used
        Response { inner: x }
    }
}

impl<'a> From<&'a str> for Response {
    fn from(val: &'a str) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(status::OK);
        let body: Body = Body(Some(val.to_string().into()));
        let x: HttpResponse<Body> = builder.body(body).unwrap(); // in the code only Ok is used
        Response { inner: x }
    }
}

impl From<::http::StatusCode> for Response {
    fn from(status: ::http::StatusCode) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(status);
        let inner = builder.body(Body(None)).unwrap();

        Response { inner }
    }
}
use hyper::Response as HResponse;
use http::Response as HttpResponse;
use std::ops::Deref;
use http::{StatusCode, HeaderMap, status};
use http::header::{HeaderValue, HeaderName};

pub struct Response {
    inner: HttpResponse<::request::RequestBody>,
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
    }

    pub fn moved_permanent<'a, T: AsRef<&'a str>>(url: T) -> Result<Response, ::http::Error> {
        use std::str::FromStr;
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

    pub fn body(&self) -> &::request::RequestBody {
        self.inner.body()
    }

    pub fn into_inner(self) -> HttpResponse<::request::RequestBody> {
        self.inner
    }
}

pub struct ResponseBuilder {
    status: StatusCode,
    body: ::request::RequestBody,
    header: HeaderMap<HeaderValue>,
}

impl ResponseBuilder {
    pub fn status<T: Into<StatusCode>>(mut self, status: T) -> Self {
        self.status = status.into();
        self
    }
    pub fn header<N: Into<HeaderName>, K: Into<HeaderValue>>(mut self, name: N, value: K) -> Self {
        self.header.insert(name.into(), value.into());
        self
    }
    pub fn header_str_value<N: Into<HeaderName>, K: Into<&str>>(mut self, name: N, value: K) -> Result<Self,::error::HttpError> {
        let value = HeaderValue::from_str(value.into())?;
        self.header.insert(name.into(), value);
        self
    }
    pub fn body<T: Into<::request::RequestBody>>(mut self, body: T) -> Self {
        self.body = body.into();
        self
    }

    pub fn build(self) -> Result<Response, ::http::Error> {
        let mut builder = HttpResponse::builder();
        builder.status(self.status);
        let mut inner = builder.body(self.body)?;
        *inner.headers_mut() = self.header;
        Ok(Response { inner })
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        ResponseBuilder { status: ::http::status::OK, body: "".into(), header: HeaderMap::new() }
    }
}

impl Deref for Response {
    type Target = HttpResponse<::request::RequestBody>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<String> for Response {
    fn from(val: String) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(status::OK);
        let x = builder.body(val.into()).unwrap(); // in the code only Ok is used
        Response { inner: x }
    }
}

impl<'a> From<&'a str> for Response {
    fn from(val: &'a str) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(status::OK);
        let body: ::request::RequestBody = val.to_string().into();
        let x: HttpResponse<::request::RequestBody> = builder.body(body).unwrap(); // in the code only Ok is used
        Response { inner: x }
    }
}

impl From<Response> for HResponse {
    fn from(res: Response) -> HResponse {
        use futures::{Future, Stream};
        let (head, body) = res.into_inner().into_parts();

        let b: ::hyper::Body = body.wait().into_inner();
        HResponse::new()
            .with_status(::hyper::StatusCode::Ok)
            .with_body(b)
    }
}

impl From<::http::StatusCode> for Response {
    fn from(status: ::http::StatusCode) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(status);
        let inner = builder.body("".into()).unwrap();

        Response { inner }
    }
}
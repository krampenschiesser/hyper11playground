// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use http::Response as HttpResponse;
use std::ops::Deref;
use http::{StatusCode, HeaderMap};
use http::header::{HeaderValue, HeaderName};
use ::body::Body;


/// wrapper type around http::Response
/// incldues the Body which is a wrapper around Option<Vec<u8>>
/// It already converts from a some types:
/// 
/// * from a String
/// * a &str
/// * a Vec<u8>
/// * a &[u8]
/// * any serde struct via ```Response::try_from_json```
/// 
#[derive(Debug)]
pub struct Response {
    inner: HttpResponse<Body>,
}

impl Response {
    /// creates a new response from a given http::Response<Body>
    pub fn from_http(inner: HttpResponse<Body>) -> Self {
        Response { inner }
    }
    /// new ResponseBuilder struct
    /// 
    /// ```
    /// extern crate http;
    /// extern crate rest_in_rust;
    /// 
    /// use rest_in_rust::*;
    /// use http::StatusCode;
    /// # #[allow(dead_code)]
    /// fn hello_world(_: &mut Request) -> Result<Response, HttpError> {
    ///     Response::builder().status(StatusCode::NOT_MODIFIED).body(Body::empty()).build()
    /// }
    /// # fn main() {}
    /// ```
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
    }
    /// converts any serde serializable struct _T_ into a valid response or an HttpError
    pub fn try_from_json<T: ::serde::Serialize>(data: T) -> Result<Self, ::error::HttpError> {
        let serialized = ::serde_json::to_string(&data)?;
        Ok(serialized.into())
    }

    /// shortcut for creating a moved permanently response
    pub fn moved_permanent<T: AsRef<str>>(url: T) -> Result<Response, ::error::HttpError> {
        let value: HeaderValue = HeaderValue::from_str(url.as_ref())?;
        Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
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

    pub fn set_header<T: AsRef<str>>(&mut self, name: ::http::header::HeaderName, value: T) -> Result<(), ::error::HttpError> {
        let value = ::http::header::HeaderValue::from_str(value.as_ref())?;
        self.inner.headers_mut().insert(name, value);
        Ok(())
    }

    /// converts the given response into a Vec<u8>, used for testing/response parsing
    pub fn into_vec(self) -> Option<Vec<u8>> {
        let (_, b) = self.into_inner().into_parts();
        b.into_inner()
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
    pub fn header_str_value<'a, N: Into<HeaderName>, T: AsRef<str>>(mut self, name: N, value: T) -> Result<Self, ::error::HttpError> {
        let value = HeaderValue::from_str(value.as_ref())?;
        self.header.insert(name.into(), value);
        Ok(self)
    }

    pub fn body_vec<T: Into<Vec<u8>>>(mut self, body: T) -> Self {
        self.body = Body(Some(body.into()));
        self
    }
    pub fn body<T: Into<Body>>(mut self, body: T) -> Self {
        self.body = body.into();
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
        ResponseBuilder { status: StatusCode::OK, body: Body(None), header: HeaderMap::new() }
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
        builder.status(StatusCode::OK);
        let x = builder.body(Body(Some(val.into()))).unwrap(); // in the code only Ok is used
        Response { inner: x }
    }
}

impl From<Vec<u8>> for Response {
    fn from(val: Vec<u8>) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(StatusCode::OK);
        let x = builder.body(Body(Some(val))).unwrap(); // in the code only Ok is used
        Response { inner: x }
    }
}

impl<'r> From<&'r [u8]> for Response {
    fn from(val: &'r [u8]) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(StatusCode::OK);
        let x = builder.body(Body(Some(val.to_vec()))).unwrap(); // in the code only Ok is used
        Response { inner: x }
    }
}

impl<'a> From<&'a str> for Response {
    fn from(val: &'a str) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(StatusCode::OK);
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
impl From<::body::Body> for Response {
    fn from(body: ::body::Body) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(StatusCode::OK);
        let inner = builder.body(body).unwrap();
        Response { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_from_str() {
        Response::builder()
            .header_str_value(::http::header::CONTENT_TYPE, "Text/CacheManifest").unwrap();
        let s: String = "huhu".into();
        Response::builder()
            .header_str_value(::http::header::CONTENT_TYPE, s).unwrap();
    }
}
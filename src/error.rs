use http::{StatusCode, HeaderMap};
use http::status;
use http::header::HeaderValue;

use ::response::Response;
//
//pub trait HttpError: Sized {
//    fn get_status(&self) -> u16;
//    fn get_msg(&self) -> &str;
//    fn get_headers(&self) -> &Headers;
//}

#[derive(Debug)]
pub struct HttpError {
    status: StatusCode,
    msg: String,
    headers: HeaderMap<HeaderValue>,
}

impl ::std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}: {}", self.status, self.msg)
    }
}

impl ::std::error::Error for HttpError {
    fn description(&self) -> &str {
        self.msg.as_str()
    }
}

impl HttpError {
    pub fn not_found<S: Into<String>>(resource: Option<S>) -> Self {
        let msg: String = resource.map(|x| x.into()).unwrap_or("".into());
        Self::internal_error(status::NOT_FOUND, msg)
    }

    pub fn bad_request<S: Into<String>>(resource: S) -> Self {
        Self::internal_error(status::BAD_REQUEST, resource)
    }

    pub fn unauthorized<S: Into<String>>(resource: S) -> Self {
        Self::internal_error(status::UNAUTHORIZED, resource)
    }

    pub fn internal_server_error<S: Into<String>>(resource: S) -> Self {
        Self::internal_error(status::INTERNAL_SERVER_ERROR, resource)
    }

    fn internal_error<S: Into<String>>(status: StatusCode, msg: S) -> Self {
        let msg: String = msg.into();
        HttpError {
            status: status,
            msg: msg,
            headers: HeaderMap::new()
        }
    }
}

impl From<HttpError> for ::http::Response<::request::Body> {
    fn from(err: HttpError) -> ::http::Response<::request::Body> {
        Response::from(err).into_inner()
    }
}

impl From<HttpError> for Response {
    fn from(err: HttpError) -> Response {
        let r = Response::builder().header_map(err.headers).status(err.status).body(err.msg.into_bytes()).build();
        match r {
            Ok(res) => res,
            Err(e) => Response::builder().status(status::INTERNAL_SERVER_ERROR).body(format!("Error happened: {:?}", e).into_bytes()).build().unwrap()
        }
    }
}

impl<'a> From<&'a str> for HttpError {
    fn from(msg: &'a str) -> Self {
        HttpError { status: ::http::status::INTERNAL_SERVER_ERROR, headers: HeaderMap::new(), msg: msg.into() }
    }
}

impl From<String> for HttpError {
    fn from(msg: String) -> Self {
        HttpError { status: ::http::status::INTERNAL_SERVER_ERROR, headers: HeaderMap::new(), msg: msg }
    }
}

impl From<::http::Error> for HttpError {
    fn from(error: ::http::Error) -> Self {
        use std::error::Error;
        HttpError::internal_server_error(error.description())
    }
}

impl From<::std::io::Error> for HttpError {
    fn from(error: ::std::io::Error) -> Self {
        use std::error::Error;
        HttpError::internal_server_error(error.description())
    }
}

impl From<::http::header::InvalidHeaderValue> for HttpError {
    fn from(error: ::http::header::InvalidHeaderValue) -> Self {
        use std::error::Error;
        HttpError::internal_server_error(error.description())
    }
}
//fixme would be awesome if this works
//impl<T: ::std::error::Error> From<T> for HttpError {
//    fn from(error: T) -> Self {
//        use std::error::Error;
//        HttpError::internal_server_error(error.description())
//    }
//}
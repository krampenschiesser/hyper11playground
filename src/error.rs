// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use http::{StatusCode, HeaderMap};
use http::header::HeaderValue;

use ::response::Response;

/// RestInRust's error type
/// Suupports various conversions out of the box
/// 
/// * conversion from any string
/// * ::http::Error
/// * ::std::io::Error
/// * ::http::header::InvalidHeaderValue
/// * ::serde_json::Error
/// * ::std::str::Utf8Error
/// * ::http::uri::InvalidUri
#[derive(Debug)]
pub struct HttpError {
    pub status: StatusCode,
    pub msg: String,
    pub headers: HeaderMap<HeaderValue>,
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
    ///Shortcut function to create a 404 not found error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        let msg: String = resource.into();
        Self::internal_error(StatusCode::NOT_FOUND, msg)
    }

    ///Shortcut function to create a 403 bad request error
    pub fn bad_request<S: Into<String>>(resource: S) -> Self {
        Self::internal_error(StatusCode::BAD_REQUEST, resource)
    }

    ///Shortcut function to create a 401 unauthorized error
    pub fn unauthorized<S: Into<String>>(resource: S) -> Self {
        Self::internal_error(StatusCode::UNAUTHORIZED, resource)
    }

    ///Shortcut function to create a 500 internal server error
    pub fn internal_server_error<S: Into<String>>(resource: S) -> Self {
        Self::internal_error(StatusCode::INTERNAL_SERVER_ERROR, resource)
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

impl From<HttpError> for ::http::Response<::body::Body> {
    fn from(err: HttpError) -> ::http::Response<::body::Body> {
        Response::from(err).into_inner()
    }
}

impl From<HttpError> for Response {
    fn from(err: HttpError) -> Response {
        let r = Response::builder().header_map(err.headers).status(err.status).body_vec(err.msg.into_bytes()).build();
        match r {
            Ok(res) => res,
            Err(e) => {
                let msg = format!("Error happened: {:?}", e);
                error!("Could not convert HttpError to response: {}", msg);
                Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body_vec(msg.into_bytes()).build().unwrap()
            }
        }
    }
}

impl<'a> From<&'a str> for HttpError {
    fn from(msg: &'a str) -> Self {
        HttpError { status: StatusCode::INTERNAL_SERVER_ERROR, headers: HeaderMap::new(), msg: msg.into() }
    }
}

impl From<String> for HttpError {
    fn from(msg: String) -> Self {
        HttpError { status: StatusCode::INTERNAL_SERVER_ERROR, headers: HeaderMap::new(), msg: msg }
    }
}

impl From<::http::Error> for HttpError {
    fn from(error: ::http::Error) -> Self {
        use std::error::Error;
        error!("Http parsing error. {}", error.description());
        HttpError::internal_server_error(error.description())
    }
}

impl From<::std::io::Error> for HttpError {
    fn from(error: ::std::io::Error) -> Self {
        use std::error::Error;
        error!("General IO error. {}", error.description());
        HttpError::internal_server_error(error.description())
    }
}

impl From<::http::header::InvalidHeaderValue> for HttpError {
    fn from(error: ::http::header::InvalidHeaderValue) -> Self {
        use std::error::Error;
        error!("Could not parse header value. {}", error.description());
        HttpError::internal_server_error(error.description())
    }
}

impl From<::serde_json::Error> for HttpError {
    fn from(error: ::serde_json::Error) -> Self {
        use std::error::Error;

        error!("Could not deserialize. {}", error.description());
        HttpError::internal_server_error(error.description())
    }
}

impl From<::std::str::Utf8Error> for HttpError {
    fn from(error: ::std::str::Utf8Error) -> Self {
        use std::error::Error;

        error!("Invalid utf8 string. {}", error.description());
        HttpError::bad_request(error.description())
    }
}

impl From<::http::uri::InvalidUri> for HttpError {
    fn from(invalid_uri: ::http::uri::InvalidUri) -> Self {
        use std::error::Error;

        error!("Invalid URI. {}", invalid_uri.description());
        HttpError::bad_request(invalid_uri.description())
    }
}

//fixme would be awesome if this works
//impl<T: ::std::error::Error> From<T> for HttpError {
//    fn from(error: T) -> Self {
//        use std::error::Error;
//        HttpError::internal_server_error(error.description())
//    }
//}
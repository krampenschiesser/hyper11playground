use http::{StatusCode, HeaderMap};
use http::status;
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
    headers: HeaderMap<String>,
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

impl From<HttpError> for ::hyper::Response {
    fn from(err: HttpError) -> ::hyper::Response {
        use hyper::{Response, Body};

        Response::new()
            .with_status(::hyper_conversion::convert_status_to_hyper(err.status))
            .with_body(Body::from(err.msg))
            .with_headers(::hyper_conversion::convert_headers_to_hyper(&err.headers))
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
        HttpError::internal_server_error(error.description())
    }
}
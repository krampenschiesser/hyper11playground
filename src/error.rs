use http::{StatusCode,HeaderMap};
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
        HttpError {
            status: status::NOT_FOUND,
            msg: msg,
            headers: HeaderMap::new()
        }
    }

    pub fn bad_url<S: Into<String>>(resource: S) -> Self {
        let msg: String = resource.into();
        HttpError {
            status: status::BAD_REQUEST,
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

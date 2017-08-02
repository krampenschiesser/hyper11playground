use hyper::{Headers,StatusCode};
//
//pub trait HttpError: Sized {
//    fn get_status(&self) -> u16;
//    fn get_msg(&self) -> &str;
//    fn get_headers(&self) -> &Headers;
//}

pub struct HttpError {
    status: StatusCode,
    msg: String,
    headers: Headers,
}

impl HttpError {
    pub fn not_found<S: Into<String>>(resource: Option<S>) -> Self {
        let msg: String = resource.map(|x| x.into()).unwrap_or("".into());
        HttpError {
            status: StatusCode::NotFound,
            msg: msg,
            headers: Headers::new()
        }
    }

    pub fn bad_url<S: Into<String>>(resource: S) -> Self {
        let msg: String = resource.into();
        HttpError {
            status: StatusCode::BadRequest,
            msg: msg,
            headers: Headers::new()
        }
    }
}

impl From<HttpError> for ::hyper::Response {
    fn from(err: HttpError) -> ::hyper::Response {
        use hyper::{Response,Body};

        Response::new().with_status(err.status).with_body(Body::from(err.msg)).with_headers(err.headers)
    }
}

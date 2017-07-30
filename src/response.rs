use hyper::Response as HResponse;
use hyper::Body as HBody;
use hyper::StatusCode;

pub struct Response {
    body: Body,
}

struct Body {
    bytes: Vec<u8>,
}

impl From<String> for Body {
    fn from(val: String) -> Self {
        Body { bytes: val.into() }
    }
}

impl<'a> From<&'a str> for Body {
    fn from(val: &'a str) -> Self {
        Body { bytes: val.into() }
    }
}

impl From<String> for Response {
    fn from(val: String) -> Self {
        Response { body: val.into() }
    }
}

impl<'a> From<&'a str> for Response {
    fn from(val: &'a str) -> Self {
        Response { body: val.into() }
    }
}

impl From<Response> for HResponse {
    fn from(res: Response) -> HResponse {
        HResponse::new()
            .with_status(StatusCode::Ok)
            .with_body(res.body)
    }
}

impl From<Body> for HBody {
    fn from(body: Body) -> HBody {
        HBody::from(body.bytes)
    }
}
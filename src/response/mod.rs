use hyper::Response as HResponse;
use hyper::Body as HBody;
use http::Response as HttpResponse;
use std::ops::Deref;
use http::{StatusCode, HeaderMap, status};
use http::header::HeaderValue;

pub struct Response {
    inner: HttpResponse<::request::RequestBody>,
}

impl Response {
    fn status(&self) -> StatusCode {
        self.inner.status()
    }
    fn headers(&self) -> &HeaderMap<HeaderValue> {
        self.inner.headers()
    }

    fn body(&self) -> &::request::RequestBody {
        self.inner.body()
    }

    fn into_inner(self) -> HttpResponse<::request::RequestBody> {
        self.inner
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
        let x = builder.body(val.into()).unwrap();// in the code only Ok is used
        Response { inner: x }
    }
}

impl<'a> From<&'a str> for Response {
    fn from(val: &'a str) -> Self {
        let mut builder = HttpResponse::builder();
        builder.status(status::OK);
        let body: ::request::RequestBody = val.to_string().into();
        let x: HttpResponse<::request::RequestBody> = builder.body(body).unwrap();// in the code only Ok is used
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
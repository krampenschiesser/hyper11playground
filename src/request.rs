use route_recognizer::Params;
use url::{Url, ParseError};

pub struct Request {
    hyper_req: ::hyper::Request,
    params: Params,
    url: Url,
}


impl Request {
    pub fn new(hyper_req: ::hyper::Request, params: Params) -> Result<Self, ParseError> {
        let url = Url::parse(hyper_req.uri().as_ref())?;
        Ok(Request { hyper_req, params, url: url })
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.find(name)
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn query(&self, name: &str) -> Option<&str> {
        ::url::form_urlencoded::parse(name);
//       self.url.query_pairs().find(|p| p.0 == name)
        None
    }
}

impl AsMut<Request> for Request {
    fn as_mut(&mut self) -> &mut Request {
        self
    }
}

impl Default for Request {
    fn default() -> Self {
        use hyper::Method;
        use hyper::Uri;

        let req = ::hyper::Request::new(Method::Get, Uri::default());
        let para = Params::new();
        Request::new(req, para).unwrap()
    }
}
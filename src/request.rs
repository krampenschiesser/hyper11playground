use route_recognizer::Params;

pub struct Request {
    hyper_req: ::hyper::Request,
    params: Params,
}


impl Request {
    pub fn new(hyper_req: ::hyper::Request, params: Params) -> Self {
        return Request { hyper_req,params };
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        None
    }

    pub fn params(&self) -> &Params {
        &self.params
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

        let req = ::hyper::Request::new(Method::Get,Uri::default());
        let para = Params::new();
        Request::new(req,para)
    }
}
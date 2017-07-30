
pub struct Request {
    hyper_req: ::hyper::Request
}



impl Request {
    pub fn new(hyper_req: ::hyper::Request) -> Self {
        return Request {hyper_req};
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        None
    }
}

impl AsMut<Request> for Request {
    fn as_mut(&mut self) -> &mut Request {
        self
    }
}
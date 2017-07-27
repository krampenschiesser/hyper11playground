pub struct Request {}

impl Request {
    pub fn new() -> Self {
        return Request {};
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
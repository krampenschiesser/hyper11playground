
pub struct Request {}

impl Request {
    pub fn new() -> Self{
        return Request{};
    }
}

impl AsMut<Request> for Request {
    fn as_mut(&mut self) -> &mut Request {
        self
    }
}
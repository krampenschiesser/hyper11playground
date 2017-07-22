use hyper::Headers;

pub struct HttpError {
    status: u16,
    msg: String,
    headers: Headers,
}

impl ::TResponse for HttpError {
    fn get_status(&self) -> u16 {
        self.status
    }

    fn get_msg(&self) -> &str {
        self.msg.as_str()
    }

    fn get_headers(&self) -> &::hyper::Headers {
        &self.headers
    }
}
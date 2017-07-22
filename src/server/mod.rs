use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

pub struct Server {}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = ::hyper::Error;
    type Future = ::futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, _req: Request) -> Self::Future {
        ::futures::future::ok(
            Response::new()
                .with_header(ContentLength("hello".len() as u64))
                .with_body("hello")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_server() {
        use hyper::server::Http;
        let addr = "127.0.0.1:3000".parse().unwrap();
        let server = Http::new().bind(&addr, || Ok(Server{})).unwrap();
        server.run().unwrap();
    }
}

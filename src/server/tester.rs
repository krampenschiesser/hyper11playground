use http::{Request, Response};
use state::Container;
use std::sync::Arc;


use body::Body;
use router::Router;
use error::HttpError;

pub struct ServerTester {
    pub router: Arc<Router>,
    pub state: Arc<Container>,
}


impl ServerTester {
    pub fn new(router: Arc<Router>, state: Arc<Container>) -> Self {
        ServerTester { router, state }
    }
    pub fn handle(&self, req: Request<Body>) -> Response<Body> {
        use request::Request as RestRequest;

        let o = self.router.resolve(req.method(), req.uri().path());
        let (route, param) = match o {
            Some(val) => val,
            None => return HttpError::not_found(Some(req.uri().path())).into(),
        };

        let mut r = RestRequest::new(req, &self.state, param.into());

        let response: ::response::Response = match route.callback.handle(&mut r) {
            Ok(response) => response,
            Err(err) => err.into(),
        };
        response.into_inner()
    }
}
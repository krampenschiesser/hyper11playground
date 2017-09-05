// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use http::{Request, Response};
use state::Container;
use std::sync::Arc;


use body::Body;
use router::InternalRouter;
use error::HttpError;

pub struct ServerTester {
    router: Arc<InternalRouter>,
    state: Arc<Container>,
}


impl ServerTester {
    pub fn new(router: Arc<InternalRouter>, state: Arc<Container>) -> Self {
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
// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use request::Request;
use error::HttpError;
use response::Response;

pub trait Handler: Send + Sync + 'static
{
    fn handle(&self, req: &mut Request) -> Result<Response, HttpError>;
}

impl<F> Handler for F
    where F: Send + Sync + 'static + Fn(&mut Request) -> Result<Response, HttpError>,
{
    fn handle(&self, req: &mut Request) -> Result<Response, HttpError> {
        (*self)(req)
    }
}

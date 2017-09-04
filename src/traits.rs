// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use ::request::Request;
use ::error::HttpError;

pub trait FromRequest: Sized {
    fn from_req(req: &mut Request) -> Result<Self, HttpError>;
}
pub trait FromRequestAsRef<'a>: Sized {
    fn from_req_as_ref(req: &'a mut Request) -> Result<&'a Self, HttpError>;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    struct Bla();

    impl FromRequest for Bla {
        fn from_req(_: &mut Request) -> Result<Self, HttpError> {
            unimplemented!()
        }
    }
    impl<'a> FromRequestAsRef<'a> for Bla {
        fn from_req_as_ref(_: &'a mut Request) -> Result<&'a Self, HttpError> {
            unimplemented!()
        }
    }
}
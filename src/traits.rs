// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines helper traits that help you to streamline your request handling

use ::request::Request;
use ::error::HttpError;

/// Helper trait that you can implement for your objects that are being parsed from a request
/// Eg. parsing queryparams to a serch struct
/// 
///```ignore
/// #[macro_use] extern crate serde_derive;
/// extern crate rest_in_rust;
/// 
/// use rest_in_rust::*;
/// 
/// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// struct MyStruct {
///     text: String,
/// }
/// impl FromRequest for MyStruct {
///     fn from_req(req: &mut Request) -> Result<Self, HttpError> {
///         req.body().to_json()
///     }
/// }
///```
pub trait FromRequest: Sized {
    /// Creates a new Self from the request, or returns an error 
    fn from_req(req: &mut Request) -> Result<Self, HttpError>;
}


/// Helper trait that you can implement for your objects that are being parsed from a request
/// use this if your object is still owned by the request, eg. for state
/// 
/// ```
/// # #[allow(unused)]
/// # use rest_in_rust::*;
/// use rest_in_rust::FromRequestAsRef;
/// 
/// # #[allow(unused)]
/// struct MyState {
///     test: String,
/// } 
/// impl<'a> FromRequestAsRef<'a> for MyState {
///     fn from_req_as_ref(req: &'a mut Request) -> Result<&'a Self, HttpError> {
///         let state: Option<&MyState> = req.get_state();
///         match state {
///            Some(state) => Ok(state),
///            None => Err("No global state present".into())
///         }
///     }
/// }
/// # #[allow(dead_code)]
/// fn handle(req: &mut Request) -> Result<Response, HttpError> {
///     let state = MyState::from_req_as_ref(req)?;
///     Ok(state.test.clone().into())
/// }
/// ```
pub trait FromRequestAsRef<'a>: Sized {
    /// Creates a new &Self from the request, or returns an error 
    fn from_req_as_ref(req: &'a mut Request) -> Result<&'a Self, HttpError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[allow(dead_code)]
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct MyStruct {
         text: String,
    }
    
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

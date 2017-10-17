// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! contains the body type, a wrapper around ```Option<Vec<u8>>```

use std::ops::Deref;
use error::HttpError;
use std::fmt::{Formatter, Result as FmtResult, Debug};


///Body used by rest in rust.
///basically a placeholder for ```Option<Vec<u8>>```
pub struct Body(pub Option<Vec<u8>>);

impl Debug for Body {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use std::str::from_utf8;

        match &self.0 {
            &None => write!(f, "Body(None)"),
            &Some(ref v) => {
                match from_utf8(v.as_ref()) {
                    Ok(s) => write!(f, "Body('{}')", s),
                    Err(_) => write!(f, "Body({:?})", v),
                }
            }
        }
    }
}

impl Body {
    /// Shortcut for creating an empty body
    /// 
    /// ```
    /// # use rest_in_rust::*;
    /// # #[allow(dead_code)]
    /// fn empty(_: &mut Request) -> Result<Response, HttpError> {
    ///     Ok(Body::empty().into())
    /// }
    /// ```
    pub fn empty() -> Self {
        ().into()
    }
    
    ///converts any serde serializeable object into a string and creates a corresponding body from it
    /// 
    /// ```
    /// extern crate rest_in_rust;
    /// #[macro_use] extern crate serde_derive;
    /// use rest_in_rust::*;
    /// 
    /// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    /// struct MyStruct {
    ///     text: String,
    /// }
    /// # #[allow(dead_code)]
    /// fn empty(_: &mut Request) -> Result<Response, HttpError> {
    ///     let my = MyStruct { text: "hello".into() };
    ///     Ok(Body::from_serde(my)?.into())
    /// }
    /// # fn main() {}
    /// ```
    pub fn from_serde<T: ::serde::Serialize>(value: T) -> Result<Self, HttpError> {
        use std::convert::TryFrom;

        Body::try_from(value)
    }
    /// get mutable reference to inner ```Option<Vec<u8>>```
    pub fn inner_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.0
    }
    /// reference to inner ```Option<Vec<u8>>```
    pub fn inner(&self) -> &Option<Vec<u8>> {
        &self.0
    }
    ///moves self to ```Option<Vec<u8>>```
    pub fn into_inner(self) -> Option<Vec<u8>> {
        self.0
    }
    ///Converts a string body to any serde deserializable body
    /// 
    /// ```
    /// extern crate rest_in_rust;
    /// #[macro_use] extern crate serde_derive;
    /// use rest_in_rust::*;
    /// 
    /// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    /// struct MyStruct {
    ///     text: String,
    /// }
    /// # #[allow(dead_code)]
    /// fn post_json(req: &mut Request) -> Result<Response, HttpError> {
    ///     let obj: MyStruct = req.body().to_json()?;
    /// 
    ///     Ok(format!("{:?}", obj).into())
    /// } 
    /// # fn main() {}
    /// ```
    pub fn to_json<T>(&self) -> Result<T, HttpError>
        where T: ::serde::de::DeserializeOwned {
        use serde_json::from_str;
        use serde_json::Error;

        let ref vec = match self.0 {
            None => Err(HttpError::bad_request("No body given, cannot parse as json")),
            Some(ref vec) => Ok(vec),
        }?;

        let string_value = match ::std::str::from_utf8(vec.as_ref()) {
            Err(e) => {
                error!("Could not parse {:?} as utf8 string: {}", vec, e);
                Err(HttpError::bad_request("Could not parse as utf8 string"))
            }
            Ok(val) => Ok(val),
        }?;

        let b: Result<T, Error> = from_str(string_value);
        match b {
            Ok(cmd) => Ok(cmd),
            Err(e) => {
                error!("Could not parse input as json: {:?}", e);
                Err(HttpError::bad_request("Could not parse input as json"))
            }
        }
    }

    ///Helper method to convert the body to a string
    pub fn to_string(&self) -> Result<String, HttpError> {
        let vec: &Vec<u8> = match self.0 {
            Some(ref v) => v,
            None => return Err(HttpError::bad_request("Trying to read non existing string from body")),
        };
        let str = ::std::str::from_utf8(vec.as_slice())?;
        Ok(str.into())
    }
}

impl Default for Body {
    fn default() -> Self {
        Body(None)
    }
}

impl Deref for Body {
    type Target = Option<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Option<Vec<u8>>> for Body {
    fn from(o: Option<Vec<u8>>) -> Self {
        Body(o)
    }
}

impl From<Vec<u8>> for Body {
    fn from(o: Vec<u8>) -> Self {
        Body(Some(o))
    }
}

impl From<String> for Body {
    fn from(o: String) -> Self {
        let v = o.into_bytes();
        Body::from(v)
    }
}

impl<'a> From<&'a str> for Body {
    fn from(o: &'a str) -> Self {
        let v = Vec::from(o.as_bytes());
        Body::from(v)
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Body(None)
    }
}

impl<T: ::serde::Serialize> ::std::convert::TryFrom<T> for Body {
    type Error = HttpError;

    fn try_from(value: T) -> Result<Self, Self::Error> {
        let string = ::serde_json::to_string(&value).map_err(|e| HttpError::from(e))?;
        Ok(Body::from(string))
    }
}
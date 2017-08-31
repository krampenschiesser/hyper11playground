use std::ops::Deref;
use error::HttpError;

#[derive(Debug)]
pub struct Body(pub Option<Vec<u8>>);

impl Body {
    pub fn inner_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.0
    }
    pub fn inner(&self) -> &Option<Vec<u8>> {
        &self.0
    }

    pub fn into_inner(self) -> Option<Vec<u8>> {
        self.0
    }

    pub fn json<T>(&self) -> Result<T, HttpError>
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
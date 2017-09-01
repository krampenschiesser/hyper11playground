use std::ops::Deref;
use error::HttpError;

#[derive(Debug)]
pub struct Body(pub Option<Vec<u8>>);

impl Body {
    pub fn empty() -> Self {
        ().into()
    }
    pub fn fom_serde<T: ::serde::Serialize>(value: T) -> Result<Self, HttpError> {
        use std::convert::TryFrom;

        Body::try_from(value)
    }

    pub fn inner_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.0
    }
    pub fn inner(&self) -> &Option<Vec<u8>> {
        &self.0
    }

    pub fn into_inner(self) -> Option<Vec<u8>> {
        self.0
    }

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
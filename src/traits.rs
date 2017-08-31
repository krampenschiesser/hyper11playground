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
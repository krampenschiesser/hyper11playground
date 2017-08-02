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

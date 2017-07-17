use hyper::Method;
use ::Handler;
use route_recognizer::Router as Recoginzer;
use std::collections::HashMap;

pub struct Router {
    routes: HashMap<Method, Recoginzer<Route>>,
}

impl Router {
    pub fn new() -> Self {
        Router { routes: HashMap::new() }
    }

    pub fn get<P: AsRef<str>, H: Handler>(&self, path: P, h: H) {}
}

pub struct Route {
    path: String,
    method: Method,
    callback: Box<Handler>
}


#[cfg(test)]
mod tests {
    use super::*;
    use ::prelude::*;

    fn handle(req: &mut Request) -> Result<Response, HttpError> {
        return Ok(Response {});
    }

    #[test]
    fn token_eq() {
        let mut router = Router::new();

        router.get("/hello", handle);


    }
}
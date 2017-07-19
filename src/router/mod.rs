use hyper::Method;
use handler::Handler;
use route_recognizer::Router as Recoginzer;
use std::collections::HashMap;

pub struct Router {
    routes: HashMap<Method, Recoginzer<Route>>,
}

impl Router {
    pub fn new() -> Self {
        Router { routes: HashMap::new() }
    }

    pub fn get<P: AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        let mut r = Recoginzer::new();
        r.add(path.as_ref(),h);

        Route{

        }
        self.routes.insert(Method::Get,r);
    }

    pub fn resolve<S: AsRef<str>, T: Handler>(&self, method: Method, path: S) -> Option<Box<T>> {
        None
    }
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
    use hyper::Method::*;
    use std::sync::Mutex;
    use std::boxed::Box;

    fn handle(req: &mut Request) -> Result<Response, HttpError> {
        return Ok(Response {});
    }

    struct HandlerStruct {
        called: Mutex<bool>
    }

    impl Default for HandlerStruct {
        fn default() -> Self {
            HandlerStruct { called: Mutex::new(false) }
        }
    }

    impl ::handler::Handler for HandlerStruct {
        fn handle(&mut self, req: &mut Request) -> Result<Response, HttpError> {
            let mut r = self.called.lock().unwrap();
            *r = true;
            Ok(Response {})
        }
    }

    impl HandlerStruct {
        pub fn get(&self) -> bool {
            let r = self.called.lock();
            *(r.unwrap())
        }
    }

    #[test]
    fn compile_handle_call() {
        let mut router = Router::new();
        router.get("/", handle);
    }

    #[test]
    fn get_resolution() {
        let mut router = Router::new();
        let handler = HandlerStruct::default();

        router.get("/hello", handler);

        let r: Option<Box<HandlerStruct>> = router.resolve(Get, "/helloNone");
        assert!(r.is_none());

        let mut handler: Box<HandlerStruct> = router.resolve(Get, "/helloNone").unwrap();
        let mut r = Request::new();
        handler.handle(r.as_mut());

        assert!(handler.get());
    }
}
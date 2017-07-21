use hyper::Method;
use handler::Handler;
use route_recognizer::Router as Recognizer;
use std::collections::HashMap;

pub struct Router {
    routes: HashMap<Method, Recognizer<Route>>,
}

impl Router {
    pub fn new() -> Self {
        Router { routes: HashMap::new() }
    }

    pub fn add<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, method: Method, path: P, h: H) {
        let path = path.into();
        let route = Route {
            path: path.clone(),
            callback: Box::new(h),
            method: method.clone()
        };

        self.routes.entry(method.clone()).or_insert(Recognizer::new()).add(path.as_ref(), route);
    }

    pub fn get<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::Get, path, h)
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

impl Route {
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }
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
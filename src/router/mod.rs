use hyper::Method;
use handler::Handler;
use route_recognizer::Router as Recognizer;
use route_recognizer::Params;
use std::collections::HashMap;
use std::cell::{RefMut,RefCell};
use std::ops::DerefMut;

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
            callback: RefCell::new(Box::new(h)),
            method: method.clone()
        };

        self.routes.entry(method.clone()).or_insert(Recognizer::new()).add(path.as_ref(), route);
    }

    pub fn get<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::Get, path, h)
    }

    pub fn resolve<S: AsRef<str>>(&self, method: &Method, path: S) -> Option<(&Route, Params)> {
        if let Some(found) = self.routes.get(method) {
            match found.recognize(path.as_ref()) {
                Ok(matching) => {
                    Some((matching.handler, matching.params))
                }
                Err(msg) => {
                    warn!("Found no handler for {} {}", method, path.as_ref());
                    None
                }
            }
        } else {
            None
        }
    }
}

pub struct Route {
    pub path: String,
    pub method: Method,
    pub callback: RefCell<Box<Handler>>
}

impl Route {
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    pub fn get_callback(&self) -> RefMut<Box<Handler>> {
        let mut b: RefMut<Box<Handler>> = self.callback.borrow_mut();
        b
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
        return Ok("bla".into());
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
            Ok("".into())
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

        let r = router.resolve(&Get, "/helloNone");
        assert!(r.is_none());

        let (route, params) = router.resolve(&Get, "/hello").unwrap();
        let mut handler: RefMut<Box<Handler>> = route.get_callback();
        let mut r = Request::new();
        (**handler).handle(r.as_mut());
    }

    #[test]
    fn parse_parameter() {
        let mut router = Router::new();

        router.get("/hello/wild/*card", HandlerStruct::default());
        router.get("/hello/:param1", HandlerStruct::default());
        router.get("/hello/:param1/bla/:param2", HandlerStruct::default());

        assert!(router.resolve(&Get, "/hello").is_none());
        has_param(router.resolve(&Get, "/hello/val1").unwrap().1, "param1", "val1");
        has_param(router.resolve(&Get, "/hello/val1/bla/val2").unwrap().1, "param1", "val1");
        has_param(router.resolve(&Get, "/hello/val1/bla/val2").unwrap().1, "param2", "val2");
        has_param(router.resolve(&Get, "/hello/wild/schrott/more").unwrap().1, "card", "schrott/more");
    }

    fn has_param(p: Params, name: &str, expected: &str) {
        let val = p.find(name).unwrap();
        assert_eq!(expected, val);
    }
}
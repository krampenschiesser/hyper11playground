use http::Method;
use handler::Handler;
use route_recognizer::Router as Recognizer;
use route_recognizer::Params;
use std::collections::HashMap;
use std::sync::Arc;

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
            callback: Arc::new(Box::new(h)),
            method: method.clone()
        };

        self.routes.entry(method.clone()).or_insert(Recognizer::new()).add(path.as_ref(), route);
    }

    pub fn get<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::GET, path, h)
    }
    pub fn put<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::PUT, path, h)
    }
    pub fn post<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::POST, path, h)
    }
    pub fn delete<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::DELETE, path, h)
    }
    pub fn options<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::OPTIONS, path, h)
    }
    pub fn head<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::HEAD, path, h)
    }
    pub fn patch<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) {
        self.add(Method::PATCH, path, h)
    }

    pub fn resolve<S: AsRef<str>>(&self, method: &Method, path: S) -> Option<(&Route, Params)> {
        if let Some(found) = self.routes.get(method) {
            match found.recognize(path.as_ref()) {
                Ok(matching) => {
                    Some((matching.handler, matching.params))
                }
                Err(msg) => {
                    warn!("Found no handler for {} {}: {}", method, path.as_ref(), msg);
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
    pub callback: Arc<Box<Handler>>
}

impl Route {
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    pub fn get_callback(&self) -> &Box<Handler> {
        &self.callback
    }
}


#[cfg(test)]
mod tests {
    use http::Request as HttpRequest;
    use super::*;
    use ::body::Body;
    use http::Method;
    use std::sync::Mutex;
    use ::*;

    fn handle(_: &mut Request) -> Result<Response, HttpError> {
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
        fn handle(&self, _: &mut Request) -> Result<Response, HttpError> {
            let mut r = self.called.lock().unwrap();
            *r = true;
            Ok("".into())
        }
    }
    //
    //    impl HandlerStruct {
    //        pub fn get(&self) -> bool {
    //            let r = self.called.lock();
    //            *(r.unwrap())
    //        }
    //    }

    #[test]
    fn compile_handle_call() {
        let mut router = Router::new();
        router.get("/", handle);
    }

    #[test]
    fn get_resolution() {
        use super::super::request::Params;

        let mut router = Router::new();
        let handler = HandlerStruct::default();

        router.get("/hello", handler);

        let r = router.resolve(&Method::GET, "/helloNone");
        assert!(r.is_none());

        let (route, _) = router.resolve(&Method::GET, "/hello").unwrap();
        let ref handler = route.get_callback();

        let req = request(Method::GET, ::http::Uri::default());
        let c = ::state::Container::new();
        let mut r = Request::new(req, &c, Params::new());
        (*handler).handle(&mut r).unwrap();
    }

    fn request(method: ::http::Method, uri: ::http::Uri) -> HttpRequest<Body> {
        let mut req = ::http::Request::new(None.into());
        *req.method_mut() = method;
        *req.uri_mut() = uri;
        req
    }

    #[test]
    fn parse_parameter() {
        let mut router = Router::new();

        router.get("/hello/wild/*card", HandlerStruct::default());
        router.get("/hello/:param1", HandlerStruct::default());
        router.get("/hello/:param1/bla/:param2", HandlerStruct::default());

        assert!(router.resolve(&Method::GET, "/hello").is_none());
        has_param(router.resolve(&Method::GET, "/hello/val1").unwrap().1, "param1", "val1");
        has_param(router.resolve(&Method::GET, "/hello/val1/bla/val2").unwrap().1, "param1", "val1");
        has_param(router.resolve(&Method::GET, "/hello/val1/bla/val2").unwrap().1, "param2", "val2");
        has_param(router.resolve(&Method::GET, "/hello/wild/schrott/more").unwrap().1, "card", "schrott/more");
    }

    fn has_param(p: Params, name: &str, expected: &str) {
        let val = p.find(name).unwrap();
        assert_eq!(expected, val);
    }

    #[test]
    fn hello_world_test() {
        let mut router = Router::new();
        router.get("/hello/:hello", HandlerStruct::default());
        has_param(router.resolve(&Method::GET, "/hello/val1").unwrap().1, "hello", "val1");
    }
}
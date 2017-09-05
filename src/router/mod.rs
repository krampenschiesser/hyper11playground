// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use http::Method;
use handler::Handler;
use route_recognizer::Router as Recognizer;
use route_recognizer::Params;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Router {
    intial: HashMap<(Method, String), Route>,
}

pub struct InternalRouter {
    routes: HashMap<Method, Recognizer<Arc<Route>>>,
}

impl InternalRouter {
    pub fn new(router: Router) -> Self {
        let mut r = InternalRouter { routes: HashMap::new() };

        for (key, route) in router.intial.into_iter() {
            r.routes.entry(key.0.clone()).or_insert(Recognizer::new()).add(key.1.as_ref(), Arc::new(route));
        }
        r
    }

    pub fn resolve<S: AsRef<str>>(&self, method: &Method, path: S) -> Option<(Arc<Route>, Params)> {
        if let Some(found) = self.routes.get(method) {
            match found.recognize(path.as_ref()) {
                Ok(matching) => {
                    Some((matching.handler.clone(), matching.params))
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

impl Router {
    pub fn new() -> Self {
        Router { intial: HashMap::new() }
    }

    pub fn add<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, method: Method, path: P, h: H) -> &mut Route {
        let path = path.into();
        let route = Route {
            path: path.clone(),
            callback: Arc::new(Box::new(h)),
            method: method.clone(),
            async: match method {
                Method::GET | Method::OPTIONS => Threading::SAME,
                _ => Threading::SEPERATE,
            },
        };

        self.intial.insert((method.clone(), path.clone()), route);
        self.intial.get_mut(&(method.clone(), path.clone())).unwrap()
    }

    pub fn get<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::GET, path, h)
    }
    pub fn put<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::PUT, path, h)
    }
    pub fn post<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::POST, path, h)
    }
    pub fn delete<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::DELETE, path, h)
    }
    pub fn options<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::OPTIONS, path, h)
    }
    pub fn head<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::HEAD, path, h)
    }
    pub fn patch<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::PATCH, path, h)
    }
}

pub struct Route {
    pub path: String,
    pub method: Method,
    pub callback: Arc<Box<Handler>>,
    pub async: Threading,
}

pub enum Threading {
    SAME,
    SEPERATE
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

        let router = InternalRouter::new(router);
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
        let router = InternalRouter::new(router);

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
        let router = InternalRouter::new(router);
        has_param(router.resolve(&Method::GET, "/hello/val1").unwrap().1, "hello", "val1");
    }
}
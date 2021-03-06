// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Contains the router you use to configure get/put/post/delete/head/options routes
//! Alsos contains routes and static file handler + cache.

use http::Method;
use handler::Handler;
use route_recognizer::Router as Recognizer;
use route_recognizer::Params;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use self::staticfile::StaticFileCache;


mod staticfile;

pub use self::staticfile::{ChangeDetection, EvictionPolicy};

/// Basic struct containing route registrations
/// when creating a server out of this, it will be converted to InternalRouter 
pub struct Router {
    static_file_cache: Arc<StaticFileCache>,
    intial: Vec<Route>,
}

/// internal router representation used by RestInRust, modifcations are no longer possible
pub struct InternalRouter {
    //    static_file_cache: Arc<StaticFileCache>,
    routes: HashMap<Method, Recognizer<Arc<Route>>>,
}

impl InternalRouter {
    pub fn new(router: Router) -> Self {
        let mut r = InternalRouter { routes: HashMap::new() };// static_file_cache: router.static_file_cache

        for route in router.intial.into_iter() {
            let method = route.method.clone();
            let path = route.path.clone();
            r.routes.entry(method).or_insert(Recognizer::new()).add(path.as_ref(), Arc::new(route));
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
    /// creates a new empty router
    pub fn new() -> Self {
        Router { intial: Vec::new(), static_file_cache: Arc::new(StaticFileCache::new()) }
    }

    /// configures the static file cache size in bytes.
    /// The default is 50_000_000 bytes/octets
    pub fn set_static_file_cache_size(&mut self, size: usize) {
        self.static_file_cache = Arc::new(StaticFileCache::with_max_size(size));
    }

    /// adds a new route to the router, please note the shortcut methods below.
    /// Additionally it returns a mutable reference to the Route which you can use to further modify the route 
    pub fn add<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, method: Method, path: P, h: H) -> &mut Route {
        let path = path.into();
        let route = Route {
            path: path.clone(),
            callback: Arc::new(Box::new(h)),
            method: method.clone(),
            threading: match method {
                Method::GET | Method::OPTIONS => Threading::SAME,
                _ => Threading::SEPERATE,
            },
        };


        self.intial.push(route);
        let index = self.intial.len() - 1;
        self.intial.get_mut(index).unwrap()
    }

    /// register a get handler
    pub fn get<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::GET, path, h)
    }
    
    /// register a put handler
    pub fn put<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::PUT, path, h)
    }
    /// register a post handler
    pub fn post<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::POST, path, h)
    }
    /// register a delete handler
    pub fn delete<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::DELETE, path, h)
    }
    /// register an options handler
    pub fn options<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::OPTIONS, path, h)
    }
    /// register an head handler
    pub fn head<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::HEAD, path, h)
    }
    /// register an patch handler
    pub fn patch<P: Into<String> + Sized + AsRef<str>, H: Handler>(&mut self, path: P, h: H) -> &mut Route {
        self.add(Method::PATCH, path, h)
    }

    /// Registers a static file path for file serving. 
    /// This can be a file or directory
    /// this path will never be cached and always read again from the file system.
    /// If you need a different behavior use ```static_path_cached```
    pub fn static_path<R, P>(&mut self, url_path: R, file_path: P) -> &mut Route
        where R: Into<String> + Sized + AsRef<str>, P: Into<PathBuf> {
        self.static_path_cached(url_path, file_path, ChangeDetection::NoCache, EvictionPolicy::Never)
    }
    /// Registers a static file path for file serving
    /// This can be a file or directory
    /// You can define how changes are discovered in order to update the internal file cache
    /// Also you can define how the cache size is kept small be defining an EvictionPolicy
    pub fn static_path_cached<R, P>(&mut self, url_path: R, file_path: P, change_detection: ChangeDetection, eviction: EvictionPolicy) -> &mut Route
        where R: Into<String> + Sized + AsRef<str>, P: Into<PathBuf> {
        let path_buf = file_path.into();
        let total_string = if path_buf.is_dir() {
            let url_string = url_path.into();
            let mut total_string = String::new();
            total_string.push_str(url_string.as_str());

            let o = url_string.as_bytes().iter().rev().next();
            let extension = match o {
                Some(b) => {
                    let x = &b'/';
                    if b == x {
                        "*file"
                    } else {
                        "/*file"
                    }
                }
                None => "/*file",
            };
            total_string.push_str(extension.as_ref());
            total_string
        } else {
            url_path.into()
        };

        let cache = self.static_file_cache.clone();
        let route = self.add(Method::GET, total_string, staticfile::StaticFileHandler::new(path_buf, cache, eviction, change_detection));
        route.threading = Threading::SEPERATE;
        route
    }
}

/// Representation of a registered route
pub struct Route {
    pub path: String,
    pub method: Method,
    pub callback: Arc<Box<Handler>>,
    /// Defines if a route is processed in the same thread as the connection handling is done
    /// or in a pooled thread
    pub threading: Threading,
}

/// Defines if a route is processed in the same thread as the connection handling is done
/// or in a pooled thread
pub enum Threading {
    /// Route is processed in the same thread as the connection 
    SAME,
    /// Route is processed in a pooled thread 
    SEPERATE
}

impl Route {
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    pub fn get_callback(&self) -> &Box<Handler> {
        &self.callback
    }
    /// returns the current thread model of the route, see Threading
    pub fn get_threading(&self) -> &Threading {
        &self.threading
    }
    
    /// Route is processed in a pooled thread 
    pub fn seperate_thread(&mut self){
        self.threading=Threading::SEPERATE
    }
    /// Route is processed in the same thread as the connection 
    pub fn same_thread(&mut self){
        self.threading=Threading::SAME
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
        let mut r = Request::new(req, Arc::new(c), Params::new());
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
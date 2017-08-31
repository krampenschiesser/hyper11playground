use std::collections::HashMap;
use state::Container;
use http::Request as HttpRequest;
use std::ops::Deref;
use ::body::Body;

mod params;

pub use self::params::Params;


#[derive(Debug)]
pub struct Request<'r> {
    inner: HttpRequest<Body>,
    state: StateHolder<'r>,
    params: Params,
    query: HashMap<String, Vec<String>>,
    remote_addr: Option<::std::net::SocketAddr>,
}

enum StateHolder<'r> {
    None,
    Some(&'r Container)
}


impl<'r> ::std::fmt::Debug for StateHolder<'r> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            &StateHolder::None => write!(f, "no state"),
            _ => write!(f, "has state"),
        }
    }
}

impl<'r> Default for Request<'r> {
    fn default() -> Self {
        Request {
            inner: HttpRequest::default(),
            state: StateHolder::None,
            params: Params::default(),
            query: HashMap::default(),
            remote_addr: None,
        }
    }
}


impl<'r> Request<'r> {
    pub fn new(req: HttpRequest<Body>, state: &'r Container, params: Params) -> Self {
        let query = Request::parse_query(req.uri().query());
        Request { inner: req, params, state: StateHolder::Some(state), query, remote_addr: None }
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name)
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn query_first(&self, name: &str) -> Option<&str> {
        let o = self.query_all().get(name);
        if let Some(vec) = o {
            if !vec.is_empty() {
                Some(vec[0].as_str())
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn query(&self, name: &str) -> Option<&Vec<String>> {
        self.query_all().get(name)
    }

    pub fn query_all(&self) -> &HashMap<String, Vec<String>> {
        &self.query
    }

    fn parse_query(query: Option<&str>) -> HashMap<String, Vec<String>> {
        use std::borrow::Borrow;

        if let Some(query) = query {
            let parsed = ::url::form_urlencoded::parse(query.as_bytes());
            let mut map = HashMap::new();
            for parse in parsed {
                let key = parse.0.borrow();
                let value = parse.1.borrow();

                map.entry(String::from(key)).or_insert(Vec::new()).push(String::from(value));
            }
            map
        } else {
            HashMap::with_capacity(0)
        }
    }

    pub fn get_state<T: Send + Sync + 'static>(&self) -> Option<&T> {
        match self.state {
            StateHolder::None => None,
            StateHolder::Some(state) => state.try_get(),
        }
    }

    pub fn set_state<T: Send + Sync + 'static>(&mut self, state: &'r Container) {
        self.state = StateHolder::Some(state);
    }

    pub fn header(&self, name: &::http::header::HeaderName) -> Option<&str> {
        let o = self.inner.headers().get(name);
        if let Some(value) = o {
            match value.to_str() {
                Ok(str) => Some(str),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn header_str(&self, name: &str) -> Option<&str> {
        use http::header::HeaderName;
        use std::str::FromStr;

        let hname = match HeaderName::from_str(name) {
            Ok(n) => n,
            Err(e) => {
                warn!("Could not parse header name {}: {:?}", name,e);
                return None;
            }
        };
        self.header(&hname)
    }

    //    pub fn body_as_str(&self) -> &str {
    //        use futures::Stream;
    //        let v: Vec<u8> = self.inner.body().collect::Vec<u8>();
    //    }
}

impl<'r> Deref for Request<'r> {
    type Target = HttpRequest<Body>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Params;
    use std::str::FromStr;
    use http::{Uri, method};

    #[test]
    fn test_query_param() {
        let c = Container::new();
        let mut r = HttpRequest::new(::body::Body(None));
        *r.method_mut() = method::GET;
        *r.uri_mut() = Uri::from_str("/bla?hallo=welt&hallo=blubb").unwrap();

        let req = Request::new(r, &c, Params::new());
        assert_eq!("welt", req.query_first("hallo").unwrap());
        assert_eq!(None, req.query_first("ne"));
        assert_eq!(2, req.query("hallo").unwrap().len());
    }
}

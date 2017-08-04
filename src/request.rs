use route_recognizer::Params;
use std::collections::HashMap;

use state::Container;

pub struct Request<'r> {
    state: &'r Container,
    hyper_req: ::hyper::Request,
    params: Params,
    query: HashMap<String, Vec<String>>,
}


impl<'r> Request<'r> {
    pub fn new(hyper_req: ::hyper::Request, state: &'r Container, params: Params) -> Self {
        let query = Request::parse_query(hyper_req.query());
        Request { hyper_req, params, state, query }
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.find(name)
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
        self.state.try_get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Request as HRequest;
    use hyper::{Method, Uri};
    use route_recognizer::Params;
    use std::str::FromStr;

    #[test]
    fn test_query_param() {
        let c = Container::new();
        let hr = HRequest::new(Method::Get, Uri::from_str("/bla?hallo=welt&hallo=blubb").unwrap());
        let req = Request::new(hr, &c, Params::new());
        assert_eq!("welt", req.query_first("hallo").unwrap());
        assert_eq!(None, req.query_first("ne"));
        assert_eq!(2, req.query("hallo").unwrap().len());
    }
}

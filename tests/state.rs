extern crate rest_in_rust;
extern crate reqwest;
extern crate http;

use rest_in_rust::*;
use rest_in_rust::server::ServerStopper;
use std::sync::RwLock;

struct State {
    echo_history: RwLock<Vec<String>>,
}

impl Default for State {
    fn default() -> Self {
        State { echo_history: RwLock::new(Vec::new()) }
    }
}


fn show_history(req: &mut Request) -> Result<Response, HttpError> {
    let state: &State = req.get_state().unwrap();
    let r = state.echo_history.read().unwrap();
    Ok(r.join("\n").into())
}

fn response(req: &mut Request) -> Result<Response, HttpError> {
    let o = req.param("hello");

    let state: &State = req.get_state().unwrap();
    match o {
        Some(string) => {
            let mut lock = state.echo_history.write().unwrap();
            lock.push(string.to_string());
            Ok(string.into())
        }
        None => Ok("Please provide a path parameter.".into())
    }
}

fn shutdown(req: &mut Request) -> Result<Response, HttpError> {
    let stopper: &ServerStopper = req.get_state().unwrap();
    stopper.stop();
    Ok("Shutting down".into())
}

fn configure() -> Router {
    let mut r = Router::new();
    r.get("/:hello", response);
    r.get("/history", show_history);
    r.get("/shutdown", shutdown);
    r
}

fn setup() -> ServerStopper {
    let addr = "127.0.0.1:8091".parse().unwrap();
    let state = State::default();
    let r = configure();
    let s = Server::new(addr, r);
    s.add_state(state);
    let stopper = s.start_http_non_blocking().unwrap();
    stopper
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn state_req() {
        super::setup();
        //        let stopper = super::setup();

        let answer = get("hallo");
        assert_eq!("hallo", answer.as_str());

        get("sauerland");


        let answer = get("history");
        assert_eq!("hallo\nsauerland", answer.as_str());

        //shutdown does not work
        //        get("shutdown");
        //        ::std::thread::sleep_ms(5000);
        //        let result= ::reqwest::get("http://127.0.0.1:8091/");
        //        println!("{:?}",result);
        //        assert!(result.is_err());
    }

    fn get(path: &str) -> String {
        let url = format!("http://127.0.0.1:8091/{}", path);
        let mut response = ::reqwest::get(url.as_str()).unwrap();

        let mut answer = String::new();
        response.read_to_string(&mut answer).unwrap();
        println!("Got response {}", answer);
        answer
    }

    #[test]
    fn test_direct_testing() {
        use http::method;
        use http::request::Builder;
        use http::Uri;
        use std::str::FromStr;

        let addr = "127.0.0.1:8091".parse().unwrap();
        let state = State::default();
        let r = configure();
        let s = Server::new(addr, r);
        s.add_state(state);

        let tester = s.start_testing();

        let mut b = Builder::new();
        b.method(method::GET);
        b.uri(Uri::from_str("http://127.0.0.1:8091/hello").unwrap());
        let request = b.body(Body::from("world")).unwrap();

        let response = tester.handle(request);
        assert_eq!(200, response.status().as_u16());

        let mut b = Builder::new();
        b.method(method::GET);
        b.uri(Uri::from_str("http://127.0.0.1:8091/history").unwrap());
        let request = b.body(().into()).unwrap();
        let response = tester.handle(request);

        assert_eq!(200, response.status().as_u16());
        let body_string = response.body().to_string().unwrap();
        assert_eq!("hello", body_string.as_str());
    }
}
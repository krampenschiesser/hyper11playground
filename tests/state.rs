extern crate rest_in_rust;
extern crate reqwest;

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

fn setup() -> ServerStopper {
    let addr = "127.0.0.1:8091".parse().unwrap();
    let state = State::default();

    let mut r = Router::new();
    r.get("/:hello", response);
    r.get("/history", show_history);
    r.get("/shutdown", shutdown);

    let s = Server::new(addr, r);
    s.add_state(state);
    let stopper = s.start_http_non_blocking().unwrap();
    stopper
}

#[cfg(test)]
mod tests {
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
}
extern crate rest_in_rust;
extern crate native_tls;

use rest_in_rust::prelude::*;
use std::sync::RwLock;

struct State {
    echo_history: RwLock<Vec<String>>,
}

impl Default for State {
    fn default() -> Self {
        State{echo_history: RwLock::new(Vec::new())}
    }
}


fn show_history(req: &mut Request) -> Result<Response, HttpError> {
    let state: &State = req.get_state().unwrap();
    let r = state.echo_history.read().unwrap();
    Ok(r.join("\n").into())
}

fn response(req: &mut Request) -> Result<Response, HttpError> {
    let o= req.param("hello");

    let state: &State = req.get_state().unwrap();
    match o {
        Some(string) => {
            let mut lock = state.echo_history.write().unwrap();
            lock.push(string.to_string());
            Ok(string.into())
        },
        None => Ok("Please provide a path parameter.".into())
    }
}

fn main() {
    let addr = "127.0.0.1:8091".parse().unwrap();
    let state= State::default();

    let mut r = Router::new();
    r.get("/:hello", response);
    r.get("/history", show_history);

    let  s = Server::new(addr,r);
    s.add_state(state);
    s.start_http().unwrap();
}


#[cfg(test)]
mod tests {
    extern crate reqwest;

//    #[test]
//    fn state_req() {
//       super::main();
//    }
}
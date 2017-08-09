extern crate rest_in_rust;

use rest_in_rust::prelude::*;

fn query_param(req: &mut Request) -> Result<Response, HttpError> {
    let all_params = req.query_all();
    let mut retval = String::new();
    for (key, value) in all_params.iter() {
        retval.push_str(format!("{}\n",key).as_str());
        for string in value.iter() {
            retval.push_str(format!("\t{}\n",string).as_str());
        }
    }
    Ok(retval.into())
}

fn main() {
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.get("/", query_param);


    let s = Server::new(addr,r);
    s.start_http_blocking().unwrap();
}
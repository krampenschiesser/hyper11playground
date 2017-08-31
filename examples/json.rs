extern crate rest_in_rust;
extern crate env_logger;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use rest_in_rust::*;

#[derive(Serialize, Deserialize, Debug)]
struct Hello {
    world: String,
}

fn post_json(req: &mut Request) -> Result<Response, HttpError> {
    let obj: Hello = req.body().json()?;

    Ok(format!("{:?}", obj).into())
}

fn get_json(_: &mut Request) -> Result<Response, HttpError> {
    let obj = Hello { world: "Hello Sauerland".into() };
    let serialized = serde_json::to_string(&obj).unwrap();
    Ok(serialized.into())
}

fn main() {
    let _ = env_logger::init();
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.post("/", post_json);
    r.get("/", get_json);

    let s = Server::new(addr, r);
    s.start_http();
}
# Rest in Rust 

[![Build Status](https://travis-ci.org/krampenschiesser/rest_in_rust.svg?branch=master)](https://travis-ci.org/krampenschiesser/rest_in_rust)
[![Build status](https://ci.appveyor.com/api/projects/status/ja363kyineu6rpiv?svg=true)](https://ci.appveyor.com/project/krampenschiesser/rest-in-rust)

Rest in rust is a very basic http 1 rest server based on 
[Tokio](https://tokio.rs/) and the [http](https://github.com/carllerche/http) crate.

It's main goal is to provide a good developer experience for easy development.
This is done by returning results from handlers and a lot of nice conversions
between types.
Everything is included, there are no plugins you have to find to *extend* behavior in order to get simple things done.

## Getting started
```
extern crate rest_in_rust;
extern crate env_logger;

use rest_in_rust::*;

fn hello_world(req: &mut Request) -> Result<Response, HttpError> {
    Ok(req.param("world").unwrap_or("sauerland").into())
}

fn main() {
    let _ = env_logger::init();
    let addr = "127.0.0.1:8091".parse().unwrap();

    let mut r = Router::new();
    r.get("/hello/:world", hello_world);

    let s = Server::new(addr, r);
    s.start_http();
}
```

## Features

* https
* simple routing
* JSON(serde) parsing in both ways (from body, to body)
* query params
* route params
* static file serving
* headless test mode (don't open socket)

### Missing

Currently it only supports simple get/post request parsing and state management.

Planned features in the near future are:

* Some more conveniance methods on request, eg. cookies
* GZIP compression for responses
* Multipart parsing
* _Maybe chunked encoding support_
* _Maybe http2 support via tokio-http2_

## Security

Not much about security in this crate, 
I would not recommend it for production use as standalone, 
always put it behind a reverse proxy.
However any suggestions on how to improve it are very welcome.
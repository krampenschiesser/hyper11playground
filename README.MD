# Rest in Rust 

[![Build Status](https://travis-ci.org/krampenschiesser/rest_in_rust.svg?branch=master)](https://travis-ci.org/krampenschiesser/rest_in_rust)
[![Build status](https://ci.appveyor.com/api/projects/status/ja363kyineu6rpiv?svg=true)](https://ci.appveyor.com/project/krampenschiesser/rest-in-rust)

Rest in rust is a very basic http 1 rest server based on 
[Tokio](https://tokio.rs/) and the [http](https://github.com/carllerche/http) crate.

It's main goal is to provide a good developer experience for easy development.
This is done by returning results from handlers and a lot of nice conversions
between types.
Everything is included, there are no plugins you have to find to *extend* behavior in order to get simple things done.


## Fetures

* https
* simple routing
* JSON(serde) parsing in both ways (from body, to body)
* query params
* route params
* static file serving

### Missing

Currently it only supports simple get/post request parsing and state management.

Planned features in the near future are:

* Some more conveniance methods on request, eg. cookies
* GZIP compression for responses
* Multipart parsing
* _Maybe chunked encoding support_
* _Maybe http2 support via tokio-http2_

## Secrutiy

Not much about security in this crate, 
I would not recommend it for production use as standalone, 
always put it behind a reverse proxy.
However any suggestions on how to improve it are very welcome.
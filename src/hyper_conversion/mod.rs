pub fn convert_method(method: &::hyper::Method) -> ::http::Method {
    use ::hyper::Method::*;
    match method {
        &Get => ::http::method::GET,
        &Put => ::http::method::PUT,
        &Post => ::http::method::POST,
        &Head => ::http::method::HEAD,
        &Patch => ::http::method::PATCH,
        &Connect => ::http::method::CONNECT,
        &Delete => ::http::method::DELETE,
        &Options => ::http::method::OPTIONS,
        &Trace => ::http::method::TRACE,
        &Extension(_) => unimplemented!(),
    }
}
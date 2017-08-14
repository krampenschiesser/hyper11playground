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

pub fn convert_headers(headers: &::hyper::Headers) -> ::http::HeaderMap<String> {
    use std::convert::TryFrom;

    let mut ret = ::http::HeaderMap::new();
    for item in headers.iter() {
        if let Ok(key) = ::http::header::HeaderName::try_from(item.name()) {
            let value: String = item.value_string();
            ret.insert(key, value);
        }
    }
    ret
}

pub fn convert_headers_to_hyper(headers: &::http::HeaderMap<String>) -> ::hyper::Headers {
    let mut ret = ::hyper::Headers::new();
    for (key, value) in headers.iter() {
        ret.set_raw(String::from(key.as_str()), value.clone());
    }
    ret
}

pub fn convert_status_to_hyper(status: ::http::StatusCode) -> ::hyper::StatusCode {
    ::hyper::StatusCode::try_from(status.as_u16()).unwrap()
}
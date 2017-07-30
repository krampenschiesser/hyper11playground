use hyper::Headers;
//
//pub trait HttpError: Sized {
//    fn get_status(&self) -> u16;
//    fn get_msg(&self) -> &str;
//    fn get_headers(&self) -> &Headers;
//}

pub struct HttpError {
    status: u16,
    msg: String,
    headers: Headers,
}

impl HttpError {
    pub fn not_found<S: Into<String>>(resource: Option<S>) -> Self {
        let msg: String = resource.map(|x| x.into()).unwrap_or("".into());
        HttpError {
            status: 404,
            msg: msg,
            headers: Headers::new()
        }
    }
}
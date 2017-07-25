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
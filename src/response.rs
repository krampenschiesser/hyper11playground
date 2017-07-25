pub struct Response {
    body: Body,
}

struct Body {
    text: String,
}

impl From<String> for Body {
    fn from(val: String) -> Self {
        Body{text:val}
    }
}
impl<'a> From<&'a str> for Body{
    fn from(val: &'a str) -> Self {
        Body{text: val.into()}
    }
}
impl From<String> for Response {
    fn from(val: String) -> Self {
        Response{body: val.into()}
    }
}
impl<'a> From<&'a str> for Response {
    fn from(val: &'a str) -> Self {
        Response{body: val.into()}
    }
}
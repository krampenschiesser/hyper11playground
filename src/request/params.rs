use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Params {
    data: BTreeMap<String, String>
}

impl From<::route_recognizer::Params> for Params {
    fn from(params: ::route_recognizer::Params) -> Self {
        let mut data: BTreeMap<String, String> = BTreeMap::new();
        for (key, value) in params.iter() {
            data.insert(key.into(), value.into());
        }
        Params { data }
    }
}

impl Params {
    pub fn new() -> Self {
        Params::default()
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

impl Default for Params {
    fn default() -> Self {
        Params { data: BTreeMap::new() }
    }
}
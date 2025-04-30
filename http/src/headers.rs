use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct HttpHeaders {
    pub hash_map: HashMap<String, String>,
}

impl HttpHeaders {
    pub fn new() -> Self {
        HttpHeaders {
            hash_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: &str, v: &str) {
        self.hash_map.insert(k.to_string(), v.to_string());
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        self.hash_map.get(k)
    }
}

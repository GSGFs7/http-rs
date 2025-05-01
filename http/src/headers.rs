use std::collections::{HashMap, hash_map};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct HttpHeaders {
    hash_map: HashMap<String, String>,
}

impl HttpHeaders {
    pub fn new() -> Self {
        HttpHeaders {
            hash_map: HashMap::new(),
        }
    }

    /// Insert to header
    pub fn insert(&mut self, k: &str, v: &str) {
        self.hash_map.insert(k.to_string(), v.to_string());
    }

    /// Get the value of the header
    pub fn get(&self, k: &str) -> Option<&String> {
        self.hash_map.get(k)
    }

    /// Return a hashmap iterator
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.hash_map.iter()
    }

    /// Check if the key in the header exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.hash_map.contains_key(key)
    }
}

/// Directly used in for loop
impl IntoIterator for HttpHeaders {
    type Item = (String, String);
    type IntoIter = hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.hash_map.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::test;

    #[test]
    async fn test_header_add() {
        let mut header = HttpHeaders::new();
        header.insert("Content-Type", "Unknown");

        assert!(header.contains_key("Content-Type"));
        assert_eq!(header.get("Content-Type").unwrap(), "Unknown");
    }
}

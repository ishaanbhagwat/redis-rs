use std::collections::HashMap;

pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String){
        self.map.insert(key, value);
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    pub fn del(&mut self, key: &str) -> Option<String>{
        self.map.remove(key)
    }
}
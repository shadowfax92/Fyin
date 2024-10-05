use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct Request {
    pub query: String,
    pub search_map: HashMap<String, SearchResult>,
    pub chunk_id_chunk_map: HashMap<usize, String>,
    pub chunk_id_to_search_id: HashMap<usize, String>,
}

#[derive(Clone)]
pub struct SearchResult {
    // name
    pub name: String,

    // url
    pub url: String,

    // content of the webiste
    pub content: Option<String>,
}

#[derive(Clone)]
pub struct Chunk {
    pub content: String,

    pub name: String,

    pub url: String,
}

pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

pub struct SearchHistory {
    pub id: i32,
    pub user_id: i32,
    pub query: String,
    pub timestamp: chrono::NaiveDateTime,
}

pub fn hash_string(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

impl Request {
    pub fn init(query: &str) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Request {
            query: query.to_string(),
            ..Default::default()
        }))
    }

    pub fn add_search_result(&mut self, search_result: SearchResult) {
        let url_hash = hash_string(&search_result.url);
        self.search_map.insert(url_hash, search_result);
    }

    pub fn add_webpage_content(&mut self, url: &str, content: String) {
        let url_hash = hash_string(&url);
        if let Some(search_result) = self.search_map.get_mut(&url_hash) {
            search_result.content = Some(content.to_string());
        }
    }

    pub fn add_id_to_chunk(&mut self, chunk: &str, search_result_id: &str, id: usize) {
        self.chunk_id_chunk_map.insert(id, chunk.to_string());
        self.chunk_id_to_search_id
            .insert(id, search_result_id.to_string());
    }

    pub fn get_chunks(&self, ids: Vec<usize>) -> Vec<Chunk> {
        ids.iter()
            .filter_map(|id| {
                let chunk_content = self.chunk_id_chunk_map.get(id).unwrap();
                let search_id = self.chunk_id_to_search_id.get(id).unwrap();

                Some(Chunk {
                    content: chunk_content.to_string(),
                    name: self.search_map.get(search_id).unwrap().name.clone(),
                    url: self.search_map.get(search_id).unwrap().url.clone(),
                })
            })
            .collect()
    }
}

impl std::fmt::Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "name: {}, url: {}", self.name, self.url,)
    }
}

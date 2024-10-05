use crate::data::{Request, SearchResult};
use crate::pretty_print;
use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};

const DEFAULT_BING_ENDPOINT: &str = "https://api.bing.microsoft.com/v7.0/search";
const DEFAULT_SEARXNG_ENDPOINT: &str = "https://searxng.example.com/search";
const DEFAULT_DUCKDUCKGO_ENDPOINT: &str = "https://api.duckduckgo.com/";

pub async fn fetch_web_pages(request: Arc<Mutex<Request>>, search_count: usize) -> Result<()> {
    // Construct a request
    let query = request.lock().unwrap().query.clone();
    let count_str = search_count.to_string();

    let search_engine = env::var("SEARCH_ENGINE").unwrap_or_else(|_| "bing".to_string());

    let (endpoint, params, headers) = match search_engine.as_str() {
        "searxng" => {
            let mut params = HashMap::new();
            params.insert("q", &query);
            params.insert("format", "json");
            let endpoint = env::var("SEARXNG_ENDPOINT").unwrap_or_else(|_| DEFAULT_SEARXNG_ENDPOINT.to_string());
            (endpoint, params, HeaderMap::new())
        },
        "duckduckgo" => {
            let mut params = HashMap::new();
            params.insert("q", &query);
            params.insert("format", "json");
            let endpoint = env::var("DUCKDUCKGO_ENDPOINT").unwrap_or_else(|_| DEFAULT_DUCKDUCKGO_ENDPOINT.to_string());
            (endpoint, params, HeaderMap::new())
        },
        _ => {
            let mut params = HashMap::new();
            params.insert("mkt", "en-US");
            params.insert("q", &query);
            params.insert("count", &count_str);
            let endpoint = env::var("BING_ENDPOINT").unwrap_or_else(|_| DEFAULT_BING_ENDPOINT.to_string());
            let mut headers = HeaderMap::new();
            headers.insert(
                "Ocp-Apim-Subscription-Key",
                HeaderValue::from_str(&env::var("BING_SUBSCRIPTION_KEY").unwrap())?,
            );
            (endpoint, params, headers)
        }
    };

    // Call the API
    let client = reqwest::Client::new();
    let response = client
        .get(endpoint)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let json: Value = response.json().await?;
        let mut request = request.lock().unwrap();

        pretty_print::print_yellow(&format!(
            "Search returned: {} results",
            json["webPages"]["value"].as_array().unwrap().len()
        ));

        json["webPages"]["value"]
            .as_array()
            .unwrap()
            .iter()
            .for_each(|wp| {
                request.add_search_result(SearchResult {
                    name: wp["name"].as_str().unwrap().to_string(),
                    url: wp["url"].as_str().unwrap().to_string(),
                    content: None,
                })
            });
        log::debug!(
            "JSON result from search: {}",
            serde_json::to_string_pretty(&json)?
        );
        Ok(())
    } else {
        Err(anyhow!(
            "Request failed with status code: {}",
            response.status()
        ))
    }
}

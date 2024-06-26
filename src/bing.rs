use crate::data::{Request, SearchResult};
use crate::pretty_print;
use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
// Change to Searxng 
const DEFAULT_BING_ENDPOINT: &str = "http://searxng:3000/search";

pub async fn fetch_web_pages(request: Arc<Mutex<Request>>, search_count: usize) -> Result<()> {
    // Construct a request
    let query = request.lock().unwrap().query.clone();
    let lang = "en-US";
    let count_str = search_count.to_string();

    let mut params = HashMap::new();
    params.insert("format", "json");
    params.insert("q", &query);
    params.insert("count", &count_str);

    let bing_endpoint = match env::var("BING_ENDPOINT") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => DEFAULT_BING_ENDPOINT.to_string(),
    };
    let bing_api_key = env::var("BING_SUBSCRIPTION_KEY").unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Ocp-Apim-Subscription-Key",
        HeaderValue::from_str(&bing_api_key)?,
    );

    // Call the API
    let client = reqwest::Client::new();
    let response = client
        .get(bing_endpoint)
        //.headers(headers)
        .query(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let json: Value = response.json().await?;
        let mut request = request.lock().unwrap();

        pretty_print::print_yellow(&format!(
            "Search returned: {} results",
            json["number_of_results"].as_array().unwrap().len()
        ));

        json["results"]
            .as_array()
            .unwrap()
            .iter()
            .for_each(|wp| {
                request.add_search_result(SearchResult {
                    name: wp["title"].as_str().unwrap().to_string(),
                    url: wp["url"].as_str().unwrap().to_string(),
                    content: None,
                })
            });
        log::debug!(
            "JSON result from bing: {}",
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

use crate::data::Request;
use crate::pretty_print;
use anyhow::{Error, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::{Arc, Mutex};

fn clean_text(text: &str) -> String {
    // Create a regex to match one or more whitespace characters
    let re = Regex::new(r"\s+").unwrap();
    // Replace one or more whitespace characters with a single space
    re.replace_all(text, " ").to_string()
}

fn fetch_and_extract_content(body: &str) -> Result<String> {
    let document = Html::parse_document(&body);
    let selector_p =
        Selector::parse("body p, body h1, body h2, body h3, article p, div p, span p").unwrap();

    let mut main_text = String::new();
    for element in document.select(&selector_p) {
        if let Some(text) = element.text().next() {
            main_text.push_str(&clean_text(text));
            main_text.push_str("\n");
        }
    }

    log::debug!("Extracted content: {}", main_text);

    Ok(main_text)
}

// Function to fetch content from URL.
async fn fetch_url_content(client: &Client, url: &str) -> Result<String> {
    pretty_print::print_yellow(&format!("Scraping content from URL: {}", url));
    let response = client.get(url).send().await.map_err(Error::new)?;
    let full_text = response.text().await.map_err(Error::new)?;
    let content = fetch_and_extract_content(&full_text)?;
    Ok(content)
}

fn get_urls(request: Arc<Mutex<Request>>) -> Result<Vec<String>> {
    let request = request.lock().unwrap();
    let mut urls = vec![];

    for (_, search_result) in request.search_map.iter() {
        urls.push(search_result.url.clone());
    }
    Ok(urls)
}

// Function to process a list of URLs in parallel and collect their content
pub async fn process_urls(request: Arc<Mutex<Request>>) -> Result<()> {
    let client = Arc::new(Client::new());
    // let semaphore = Arc::new(Semaphore::new(20)); // Limit to 10 concurrent requests.
    let mut tasks: FuturesUnordered<tokio::task::JoinHandle<Result<()>>> = FuturesUnordered::new();
    let urls = get_urls(request.clone())?;

    for url in urls {
        let client_ref = client.clone();
        let request_clone = request.clone();
        // let permit = semaphore.clone().acquire_owned().await.unwrap();
        tasks.push(tokio::spawn(async move {
            let webpage_content = fetch_url_content(&client_ref, &url).await;
            // TODO: handle error here
            let _ = webpage_content
                .map(|content| {
                    request_clone
                        .lock()
                        .unwrap()
                        .add_webpage_content(&url, content);
                })
                .map_err(|e| {
                    log::warn!("Failed fetching content for URL: {}, error: {}", url, e);
                });
            Ok(())
        }));
    }

    while let Some(result) = tasks.next().await {
        match result {
            Ok(_) => {}
            Err(e) => return Err(anyhow::Error::new(e)),
        }
    }

    Ok(())
}

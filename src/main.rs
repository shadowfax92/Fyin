#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

mod args;
mod data;
mod embedding;
mod llm;
mod pretty_print;
mod scraper;
mod vector;
mod search;

use anyhow::Result;
use clap::Parser;

use std::sync::Arc;
use tokio::sync;

use dotenv;
use std::env;

async fn init() -> Result<()> {
    // load ENV variables
    dotenv::dotenv().ok();

    // verify required ones are present
    let env_vars = [
        "OPENAI_API_KEY",
        "BING_SUBSCRIPTION_KEY",
        "EMBEDDING_MODEL_NAME",
        "CHAT_MODEL_NAME",
    ];

    for &var_name in &env_vars {
        assert!(
            !env::var(var_name)
                .expect(&format!("Failed to retrieve '{}'", var_name))
                .is_empty(),
            "The environment variable '{}' must be set and not empty.",
            var_name
        );
    }

    pretty_env_logger::init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init().await?;
    let args = args::Args::parse();

    prompt(&args.query, args.search).await?;
    Ok(())
}

async fn prompt(prompt: &str, search_count: usize) -> Result<()> {
    pretty_print::print_blue(&format!("Searching for: {}", prompt));
    let request = data::Request::init(prompt);
    let llm_agent = llm::LlmAgent::init().await;

    // do a test embed and figure out dimension

    let dimension = llm_agent.embed_string(prompt).await.unwrap().len();

    // create a new vector client
    let vector_client = Arc::new(sync::Mutex::new(
        vector::VectorDB::init(Some(dimension)).await?,
    ));

    // fetch search results
    pretty_print::print_blue("Fetching search results from bing...");
    search::fetch_web_pages(request.clone(), search_count).await?;

    // scrape content
    pretty_print::print_blue("Scraping content from search results...");
    scraper::process_urls(request.clone()).await?;

    // do embedding on all the scrapped contents.
    // store in vector DB
    pretty_print::print_blue("Embedding content...");
    embedding::generate_upsert_embeddings(request.clone(), vector_client.clone()).await?;

    // convert prompt to embedding
    let prompt_embedding = llm_agent.embed_string(prompt).await?;

    // build vector index
    vector_client.lock().await.build_index().await?;

    // search across embedding
    // and get all embedding ids
    let ids = vector_client
        .lock()
        .await
        .search(&prompt_embedding, 10)
        .await?;

    // get content
    let chunks: Vec<data::Chunk> = request.lock().unwrap().get_chunks(ids);

    let llm_agent = llm::LlmAgent::init().await;
    llm_agent.answer_question_stream(prompt, &chunks).await?;

    //clean-up vector DB
    vector_client.lock().await.clean_up().await?;

    Ok(())
}

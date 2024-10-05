#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

mod args;
mod bing;
mod data;
mod embedding;
mod llm;
mod pretty_print;
mod scraper;
mod vector;
mod db;

use anyhow::Result;
use clap::Parser;

use std::sync::Arc;
use tokio::sync;

use dotenv;
use std::env;

async fn init() -> Result<()> {
    dotenv::dotenv().ok();

    let env_vars = [
        "OPENAI_API_KEY",
        "BING_SUBSCRIPTION_KEY",
        "EMBEDDING_MODEL_NAME",
        "CHAT_MODEL_NAME",
        "DATABASE_URL",
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

    if let Some(username) = args.register {
        let password = rpassword::prompt_password("Password: ").unwrap();
        db::register_user(&username, &password).await?;
        println!("User registered successfully.");
        return Ok(());
    }

    if let Some(username) = args.login {
        let password = rpassword::prompt_password("Password: ").unwrap();
        if db::login_user(&username, &password).await? {
            println!("Login successful.");
            prompt(&args.query, args.search, Some(username)).await?;
        } else {
            println!("Invalid username or password.");
        }
        return Ok(());
    }

    prompt(&args.query, args.search, None).await?;
    Ok(())
}

async fn prompt(prompt: &str, search_count: usize, username: Option<String>) -> Result<()> {
    pretty_print::print_blue(&format!("Searching for: {}", prompt));
    let request = data::Request::init(prompt);
    let llm_agent = llm::LlmAgent::init().await;

    let dimension = llm_agent.embed_string(prompt).await.unwrap().len();

    let vector_client = Arc::new(sync::Mutex::new(
        vector::VectorDB::init(Some(dimension)).await?,
    ));

    pretty_print::print_blue("Fetching search results from bing...");
    bing::fetch_web_pages(request.clone(), search_count).await?;

    pretty_print::print_blue("Scraping content from search results...");
    scraper::process_urls(request.clone()).await?;

    pretty_print::print_blue("Embedding content...");
    embedding::generate_upsert_embeddings(request.clone(), vector_client.clone()).await?;

    let prompt_embedding = llm_agent.embed_string(prompt).await?;

    vector_client.lock().await.build_index().await?;

    let ids = vector_client
        .lock()
        .await
        .search(&prompt_embedding, 10)
        .await?;

    let chunks: Vec<data::Chunk> = request.lock().unwrap().get_chunks(ids);

    let llm_agent = llm::LlmAgent::init().await;
    llm_agent.answer_question_stream(prompt, &chunks).await?;

    if let Some(username) = username {
        db::save_search_history(&username, prompt).await?;
    }

    vector_client.lock().await.clean_up().await?;

    Ok(())
}

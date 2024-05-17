use crate::data::Chunk;
use crate::errors::EmbeddingError;
use anyhow::Result;
use openai::{
    chat::{ChatCompletion, ChatCompletionBuilder, ChatCompletionDelta, ChatCompletionMessage},
    embeddings::Embedding,
};

use std::env;
use tokio::sync::mpsc::Receiver;

type Conversation = Receiver<ChatCompletionDelta>;

const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

pub fn setup() -> Result<()> {
    let openai_base_url = match env::var("OPENAI_BASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => DEFAULT_OPENAI_BASE_URL.to_string(),
    };
    openai::set_base_url(openai_base_url);
    openai::set_key(env::var("OPENAI_KEY").unwrap().to_string());
    Ok(())
}

pub async fn embed_string(prompt: &str) -> Result<Embedding> {
    Embedding::create(&env::var("EMBEDDING_MODEL_NAME").unwrap(), prompt, "fyin")
        .await
        .map_err(|error| {
            log::error!("Error when embedding: {:?}", error);
            EmbeddingError {}.into()
        })
}

fn chunks_to_yaml(chunks: &Vec<Chunk>) -> String {
    let mut yaml_string = String::new();
    for (id, chunk) in chunks.iter().enumerate() {
        // Format each Chunk into the specified YAML-like format
        let chunk_yaml = format!(
            "Name: {}\nurl: {}\nfact: {}\nid: {}\n\n",
            chunk.name,
            chunk.url,
            chunk.content,
            id + 1 // id is 0-based, we want it to start from 1
        );
        // Append the formatted chunk to the overall string
        yaml_string.push_str(&chunk_yaml);
    }
    yaml_string
}

pub async fn chat_stream(query: &str, chunks: &Vec<Chunk>) -> Result<Conversation> {
    let format_chunks = chunks_to_yaml(chunks);
    let prompt = format!(
        "SOURCES:
        {}

        QUESTION:
        {}

        INSTRUCTIONS:
        Please provide a detailed answer to the question above only using the sources provide. 
        Include in-text citations like this [1] for each significant fact or statement at the end of the sentence. 
        At the end of your response, list all sources in a citation section with the format: [citation number] Name - URL.",
        format_chunks, query
    );

    ChatCompletionBuilder::default()
        .model(&env::var("CHAT_MODEL_NAME").unwrap())
        .temperature(0.0)
        .user("fyin")
        .messages(vec![ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::User,
            content: Some(prompt),
            name: Some("fyin".to_string()),
            function_call: None,
        }])
        .create_stream()
        .await
        .map_err(|_| EmbeddingError {}.into())
}

pub async fn _chat(prompt: &str, contents: &str) -> Result<ChatCompletion> {
    let content = format!("{}\n Context: {}\n Be concise", prompt, contents);

    ChatCompletionBuilder::default()
        .model(&env::var("CHAT_MODEL_NAME").unwrap())
        .temperature(0.0)
        .user("fyin")
        .messages(vec![ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::User,
            content: Some(content),
            name: Some("fyin".to_string()),
            function_call: None,
        }])
        .create()
        .await
        .map_err(|_| EmbeddingError {}.into())
}

use crate::data::Chunk;
use crate::pretty_print;
use anyhow::Result;
use langchain_rust::embedding::{Embedder, FastEmbed};
use langchain_rust::llm::OpenAIConfig;
use owo_colors::OwoColorize;

use futures_util::StreamExt;
use langchain_rust::embedding::openai::OpenAiEmbedder;
use langchain_rust::llm::openai::OpenAI;
use langchain_rust::{
    chain::{builder::ConversationalChainBuilder, Chain},
    fmt_message,
    llm::openai::OpenAIModel,
    memory::SimpleMemory,
    message_formatter, prompt_args,
    schemas::Message,
};

use futures_util::Stream;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use std::env;
use std::io::{stdout, Write};
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::wrappers::UnboundedReceiverStream;

const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

pub struct LlmAgent {
    pub openai: Option<OpenAI<OpenAIConfig>>,
    pub ollama: Option<Ollama>,
    local_mode: bool,
    embed_model: String,
    chat_model: String,
    use_fast_embed: bool,
}

use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref PRINTED: AtomicBool = AtomicBool::new(false);
}

fn print_message_once(local_mode: bool) {
    if !PRINTED.swap(true, Ordering::SeqCst) {
        if local_mode {
            pretty_print::print_yellow("Running in local mode using ollama");
        } else {
            pretty_print::print_yellow("Running using openai");
        }
    }
}

impl LlmAgent {
    pub async fn init() -> Self {
        let base_url = match env::var("OPENAI_BASE_URL") {
            Ok(value) if !value.trim().is_empty() => value,
            _ => DEFAULT_OPENAI_BASE_URL.to_string(),
        };

        let local_mode = if base_url.contains("localhost") {
            true
        } else {
            false
        };

        let key = env::var("OPENAI_API_KEY").unwrap();

        print_message_once(local_mode);

        if local_mode == true {
            LlmAgent {
                openai: None,
                local_mode,
                ollama: Some(Ollama::default()),
                embed_model: env::var("EMBEDDING_MODEL_NAME").unwrap(),
                chat_model: env::var("CHAT_MODEL_NAME").unwrap(),
                use_fast_embed: true,
            }
        } else {
            LlmAgent {
                openai: Some(
                    OpenAI::default()
                        .with_config(
                            OpenAIConfig::default()
                                .with_api_base(base_url)
                                .with_api_key(key),
                        )
                        .with_model(env::var("CHAT_MODEL_NAME").unwrap()),
                ),
                local_mode,
                ollama: None,
                embed_model: env::var("EMBEDDING_MODEL_NAME").unwrap(),
                chat_model: env::var("CHAT_MODEL_NAME").unwrap(),
                use_fast_embed: false,
            }
        }
    }

    pub async fn embed_string(&self, prompt: &str) -> Result<Vec<f64>> {
        if self.local_mode {
            if self.use_fast_embed {
                let fastembed = FastEmbed::try_new().unwrap();
                let result = fastembed
                    .embed_query(prompt)
                    .await?
                    .iter()
                    .map(|x| *x as f64)
                    .collect();
                Ok(result)
            } else {
                Ok(self
                    .ollama
                    .as_ref()
                    .unwrap()
                    .generate_embeddings(self.embed_model.to_string(), prompt.to_string(), None)
                    .await
                    .unwrap()
                    .embeddings)
            }
        } else {
            let openai = OpenAiEmbedder::default();
            let response = openai.embed_query(prompt).await?;
            Ok(response)
        }
    }

    fn chunk_to_documents(chunks: &Vec<Chunk>) -> Result<Vec<String>> {
        let mut documents = Vec::new();
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
            documents.push(chunk_yaml);
        }
        Ok(documents)
    }

    pub async fn answer_question_stream(
        &self,
        query: &str,
        chunks: &Vec<Chunk>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
        if self.local_mode {
            Ok(Box::pin(self.answer_using_ollama(query, chunks).await?))
        } else {
            Ok(Box::pin(self.answer_using_openai(query, chunks).await?))
        }
    }

    async fn answer_using_ollama(
        &self,
        query: &str,
        chunks: &Vec<Chunk>,
    ) -> Result<impl Stream<Item = Result<String, anyhow::Error>> + Send> {
        let documents = Self::chunk_to_documents(chunks)?;
        let prompt = format!(
            "
            SOURCES:
            {sources}

            QUESTION:
            {question}

            INSTRUCTIONS:
            You are a helpful AI assistant that helps users answer questions using the provided sources. If answer is not in sources, say you don't know rather than making up an answer.

            Please provide a detailed answer to the question above only using the sources provided.
            Include in-text citations like this [1] for each significant fact or statement at the end of the sentence.
            At the end of your response, list all sources in a citation section with the format: [citation number] Name - URL.
        ",
            sources = documents.join("\n"),
            question = query
        );

        let mut stream = self
            .ollama
            .as_ref()
            .unwrap()
            .generate_stream(GenerationRequest::new(self.chat_model.to_string(), prompt))
            .await
            .unwrap();

        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(res) = stream.next().await {
                let responses = res.unwrap();
                for resp in responses {
                    let _ = tx.send(Ok(resp.response.as_str().green().to_string()));
                }
            }
        });

        Ok(UnboundedReceiverStream::new(rx))
    }

    async fn answer_using_openai(
        &self,
        query: &str,
        chunks: &Vec<Chunk>,
    ) -> Result<impl Stream<Item = Result<String, anyhow::Error>> + Send> {
        let documents = Self::chunk_to_documents(chunks)?;
        let llm = OpenAI::default().with_model(OpenAIModel::Gpt35);
        let memory = SimpleMemory::new();

        let chain = ConversationalChainBuilder::new()
            .llm(llm)
            .prompt(message_formatter![
                fmt_message!(Message::new_system_message("You are a helpful AI assistant that helps users answer questions using the provided sources. If answer is not in sources, say you don't know rather than making up an answer.")),
                fmt_message!(Message::new_system_message(
                    format!(
                        "
                        SOURCES:
                        {sources}

                        QUESTION:
                        {question}

                        INSTRUCTIONS:
                        Please provide a detailed answer to the question above only using the sources provided.
                        Include in-text citations like this [1] for each significant fact or statement at the end of the sentence.
                        At the end of your response, list all sources in a citation section with the format: [citation number] Name - URL.
                    ",
                        sources = documents.join("\n"),
                        question = query
                    )
                ))
            ])
            .memory(memory.into())
            .build()
            .expect("Error building ConversationalChain");

        let input_variables = prompt_args! {
            "input" => "",
        };

        let mut stream = chain.stream(input_variables).await.unwrap();

        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(data) => {
                        let _ = tx.send(Ok(data.content.green().to_string()));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!(e)));
                    }
                }
            }
        });

        Ok(UnboundedReceiverStream::new(rx))
    }
}

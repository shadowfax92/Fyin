use crate::data::Request;
use crate::llm;

use crate::vector::VectorDB;
use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};

use std::sync::{Arc, Mutex};
use tokio::sync;
use tokio::task::JoinHandle;

static CHUNK_SIZE: usize = 1000;

async fn insert_embedding(
    vector_client: Arc<sync::Mutex<VectorDB>>,
    embedding: Vec<f64>,
    // id: &Uuid,
    id: usize,
) -> Result<()> {
    vector_client
        .lock()
        .await
        .upsert_embedding(embedding, id)
        .await?;
    Ok(())
}

pub async fn generate_upsert_embeddings(
    request: Arc<Mutex<Request>>,
    vector_client: Arc<sync::Mutex<VectorDB>>,
) -> Result<()> {
    let mut tasks: FuturesUnordered<JoinHandle<Result<()>>> = FuturesUnordered::new();

    // chunk the content into CHUNK_SIZE words
    let search_map = request.lock().unwrap().search_map.clone();
    let shared_counter = Arc::new(Mutex::new(0usize));

    for (url_hash, result) in search_map.into_iter() {
        let content = result.content.unwrap_or("".to_string()).clone();

        let chunks = content.split_whitespace().collect::<Vec<&str>>();
        let chunks = chunks
            .chunks(CHUNK_SIZE)
            .map(|chunk| chunk.join(" "))
            .collect::<Vec<String>>();

        log::info!(
            "Chunked content into {} chunks for url: {}",
            chunks.len(),
            result.url
        );

        // parallely process chunks and store it
        for chunk in chunks.into_iter() {
            let request_clone = request.clone();
            let qdrant_client_clone = vector_client.clone();
            let shared_counter_clone = shared_counter.clone();
            let url_hash_clone = url_hash.clone();
            tasks.push(tokio::spawn(async move {
                let llm_agent = llm::LlmAgent::init().await;
                let embedding = llm_agent.embed_string(&chunk).await.unwrap();


                // increment the counter
                let map_index = {
                    let mut counter = shared_counter_clone.lock().unwrap();
                    *counter += 1;
                    *counter
                };

                // store chunks to id mapping
                request_clone
                    .lock()
                    .unwrap()
                    .add_id_to_chunk(&chunk, &url_hash_clone, map_index);

                insert_embedding(qdrant_client_clone, embedding, map_index)
                    .await
                    .unwrap();
                Ok(())
            }));
        }
    }

    while let Some(result) = tasks.next().await {
        match result {
            Ok(_r) => {}
            Err(e) => return Err(anyhow::Error::new(e)),
        }
    }
    Ok(())
}

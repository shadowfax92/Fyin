use anyhow::{anyhow, Result};
use hora::core::ann_index::ANNIndex;


static DIMENSION: usize = 1536;

pub struct VectorDB {
    hora: hora::index::hnsw_idx::HNSWIndex<f64, usize>,
}

impl VectorDB {
    pub async fn init(dimension: Option<usize>) -> Result<Self> {
        let dimension = dimension.unwrap_or(DIMENSION);

        let index = hora::index::hnsw_idx::HNSWIndex::<f64, usize>::new(
            dimension,
            &hora::index::hnsw_params::HNSWParams::<f64>::default(),
        );
        let instance = VectorDB { hora: index };
        Ok(instance)
    }

    pub async fn clean_up(&self) -> Result<()> {
        Ok(())
    }

    pub async fn build_index(&mut self) -> Result<()> {
        // TODO: It crashes with CosineSimilarity metric - https://github.com/hora-search/hora/issues/40
        // Maybe move to https://github.com/instant-labs/instant-distance?tab=readme-ov-file!()

        self.hora
            .build(hora::core::metrics::Metric::Euclidean)
            .unwrap();
        Ok(())
    }

    pub async fn upsert_embedding(&mut self, embedding: Vec<f64>, id: usize) -> Result<()> {
        log::info!("Embedded: {}", id);

        self.hora
            .add(&embedding, id)
            .map_err(|e| anyhow!("Failed to add point: {:?}", e))?;
        Ok(())
    }

    pub async fn search(&self, embedding: &Vec<f64>, n: usize) -> Result<Vec<usize>> {
        let search_result = self.hora.search(embedding, n);
        Ok(search_result)
    }
}

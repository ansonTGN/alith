use crate::embeddings::{Embeddings, EmbeddingsData, EmbeddingsError};
use async_trait::async_trait;
use hnsw_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum VectorStoreError {
    #[error("Embedding error: {0}")]
    EmbeddingError(#[from] EmbeddingsError),
    /// JSON error (e.g.: serialization, deserialization, etc.)
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Datastore error: {0}")]
    DatastoreError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Missing Id: {0}")]
    MissingIdError(String),
    #[error("Search error: {0}")]
    SearchError(String),
    #[error("Custom error: {0}")]
    CustomError(String),
}

pub type TopNItem = (DocumentId, String, f32);
pub type TopNResults = Result<Vec<TopNItem>, VectorStoreError>;
pub type TopNResult = Result<TopNItem, VectorStoreError>;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
pub struct DocumentId(pub String);

impl Serialize for DocumentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

/// Trait representing a storage backend.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Saves a value into the storage.
    async fn save(&self, value: String) -> Result<(), VectorStoreError>;
    /// Searches the storage with a query, limiting the results and applying a threshold.
    async fn search(&self, query: &str, limit: usize, threshold: f32) -> TopNResults;
    /// Resets the storage by clearing all stored data.
    async fn reset(&self) -> Result<(), VectorStoreError>;
}

/// In-memory storage implementation.
pub struct InMemoryStorage<E: Embeddings> {
    data: Arc<RwLock<Vec<EmbeddingsData>>>, // Simple in-memory vector to store data.
    hnsw: Arc<RwLock<Hnsw<'static, f64, DistCosine>>>,
    embeddings: Arc<E>,
}

impl<E: Embeddings> InMemoryStorage<E> {
    /// Creates a new instance of `InMemoryStorage`.
    pub fn from_documents(embeddings: E, documents: Vec<EmbeddingsData>) -> Self {
        Self {
            hnsw: Arc::new(RwLock::new(Self::build_hnsw(&documents))),
            data: Arc::new(RwLock::new(documents)),
            embeddings: Arc::new(embeddings),
        }
    }

    /// Creates a new instance of `InMemoryStorage`.
    pub fn from_multiple_documents<T>(
        embeddings: E,
        documents: Vec<(T, Vec<EmbeddingsData>)>,
    ) -> Self {
        let documents = documents.iter().flat_map(|d| d.1.clone()).collect();
        Self::from_documents(embeddings, documents)
    }
}

#[async_trait]
impl<E: Embeddings> Storage for InMemoryStorage<E> {
    async fn save(&self, value: String) -> Result<(), VectorStoreError> {
        let mut data = self.data.write().await;
        let embeddings = self
            .embeddings
            .embed_texts(vec![value])
            .await
            .map_err(VectorStoreError::EmbeddingError)?;
        data.append(&mut embeddings.clone());
        let list: Vec<_> = embeddings
            .iter()
            .enumerate()
            .map(|(k, data)| (&data.vec, k))
            .collect();
        self.hnsw.write().await.parallel_insert(&list);
        Ok(())
    }

    async fn search(&self, query: &str, limit: usize, threshold: f32) -> TopNResults {
        // Collect the necessary data from the MutexGuard before entering the async block
        let data = self.data.read().await;
        let embeddings = self
            .embeddings
            .clone()
            .embed_texts(vec![query.to_string()])
            .await?;
        self.vector_search(embeddings, limit, threshold)
            .await
            .map(|result| {
                result
                    .iter()
                    .map(|result| {
                        (
                            result.0.clone(),
                            data[result.0.0.parse::<usize>().unwrap()].document.clone(),
                            result.1,
                        )
                    })
                    .collect::<Vec<_>>()
            })
    }

    async fn reset(&self) -> Result<(), VectorStoreError> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }
}

impl<E: Embeddings> InMemoryStorage<E> {
    pub async fn vector_search(
        &self,
        embeddings: Vec<EmbeddingsData>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<(DocumentId, f32)>, VectorStoreError> {
        let embeddings: Vec<Vec<f64>> = embeddings.iter().map(|e| e.vec.clone()).collect();
        let output: Vec<(DocumentId, f32)> = self
            .hnsw
            .read()
            .await
            .parallel_search(&embeddings, limit, 30)
            .into_iter()
            .flat_map(|list| {
                list.into_iter()
                    .filter_map(|v| {
                        let score = 1.0 - v.distance;
                        if score > threshold {
                            Some((DocumentId(v.d_id.to_string()), score))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(output)
    }

    pub fn build_hnsw(data: &[EmbeddingsData]) -> Hnsw<'static, f64, DistCosine> {
        let hnsw = Hnsw::new(32, data.len(), 16, 200, DistCosine {});
        let list: Vec<_> = data
            .iter()
            .enumerate()
            .map(|(k, data)| (&data.vec, k))
            .collect();
        hnsw.parallel_insert(&list);
        hnsw
    }
}

/// Simple in-memory vector store using cosine similarity
/// This is a temporary implementation until HNSW dependency issues are resolved
use std::collections::HashMap;
use anyhow::Result;

/// Simple vector store for context retrieval
pub struct SimpleVectorStore {
    embeddings: HashMap<usize, [f32; 128]>,
    dimension: usize,
    next_id: usize,
}

impl SimpleVectorStore {
    pub fn new(dimension: usize) -> Self {
        Self {
            embeddings: HashMap::new(),
            dimension,
            next_id: 0,
        }
    }

    /// Insert an embedding vector into the store
    pub fn insert(&mut self, embedding: &[f32]) -> Result<usize> {
        if embedding.len() != self.dimension {
            anyhow::bail!("Embedding dimension mismatch: expected {}, got {}", self.dimension, embedding.len());
        }
        
        let id = self.next_id;
        self.next_id += 1;
        
        let mut stored_embedding = [0.0f32; 128];
        stored_embedding.copy_from_slice(embedding);
        self.embeddings.insert(id, stored_embedding);
        
        Ok(id)
    }

    /// Compute cosine similarity between two vectors
    /// Returns value between -1.0 and 1.0, where 1.0 = identical
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }

    /// Query for the top-k nearest neighbors using cosine similarity
    pub fn query(&self, embedding: &[f32], k: usize) -> Result<Vec<[f32; 128]>> {
        if embedding.len() != self.dimension {
            anyhow::bail!("Embedding dimension mismatch: expected {}, got {}", self.dimension, embedding.len());
        }
        
        if self.embeddings.is_empty() {
            // Return query embedding as fallback
            let mut context = [0.0f32; 128];
            context.copy_from_slice(embedding);
            return Ok(vec![context]);
        }
        
        // Compute similarities and find top-k
        let mut similarities: Vec<(usize, f32)> = self.embeddings
            .iter()
            .map(|(id, stored)| (*id, Self::cosine_similarity(embedding, stored)))
            .collect();
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top-k embeddings
        let mut neighbors = Vec::with_capacity(k.min(similarities.len()));
        for (id, _) in similarities.iter().take(k) {
            if let Some(&stored_embedding) = self.embeddings.get(id) {
                neighbors.push(stored_embedding);
            }
        }
        
        // If we got fewer than k results, pad with query embedding
        while neighbors.len() < k {
            let mut context = [0.0f32; 128];
            context.copy_from_slice(embedding);
            neighbors.push(context);
        }
        
        Ok(neighbors)
    }

    /// Get the top-1 context embedding
    pub fn get_context(&self, embedding: &[f32; 128]) -> Result<[f32; 128]> {
        let results = self.query(embedding, 1)?;
        Ok(results[0])
    }
    
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }
}


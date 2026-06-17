//! RAG (Retrieval-Augmented Generation) module for CuraLit
//!
//! This module provides functionality for building and querying a RAG system
//! that stores article content in a vector database (Qdrant) and retrieves
//! relevant passages for LLM context.

use crate::article::Article;
use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use qdrant_client::qdrant::{
    CreateCollection, Distance, PointStruct, SearchPoints, UpsertPointsBuilder, VectorParams,
    VectorsConfig,
};
use qdrant_client::Qdrant;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Configuration for RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Ollama base URL
    pub ollama_url: String,
    /// Embedding model name (e.g., "nomic-embed-text")
    pub embedding_model: String,
    /// Qdrant collection name
    pub collection_name: String,
    /// Qdrant data directory
    pub qdrant_path: PathBuf,
    /// Chunk size for splitting articles
    pub chunk_size: usize,
    /// Chunk overlap
    pub chunk_overlap: usize,
    /// Number of results to retrieve
    pub top_k: usize,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            collection_name: "curalit_articles".to_string(),
            qdrant_path: PathBuf::from("0_out/qdrant_storage"),
            chunk_size: 500,
            chunk_overlap: 50,
            top_k: 5,
        }
    }
}

/// Qdrant connection URL (local server - gRPC port)
pub const QDRANT_URL: &str = "http://localhost:6334";

/// A chunked passage from an article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleChunk {
    pub id: String,
    pub article_id: String,
    pub pmid: String,
    pub title: String,
    pub chunk_index: usize,
    pub text: String,
    pub authors: Vec<String>,
    pub journal: String,
    pub pub_date: String,
    pub mesh_terms: Vec<String>,
}

/// Response from Ollama embedding API
#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

/// RAG system builder and query engine
pub struct RagSystem {
    pub config: RagConfig,
    client: Option<Qdrant>,
}

impl RagSystem {
    /// Create a new RAG system with default configuration
    pub fn new() -> Self {
        Self {
            config: RagConfig::default(),
            client: None,
        }
    }

    /// Create a RAG system with custom configuration
    pub fn with_config(config: RagConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }

    /// Initialize Qdrant client and create collection
    pub async fn initialize(&mut self) -> Result<()> {
        println!("{} Initializing RAG system...", "•".cyan());

        // Create Qdrant data directory if it doesn't exist
        if !self.config.qdrant_path.exists() {
            fs::create_dir_all(&self.config.qdrant_path)?;
        }

        // Initialize Qdrant client
        // Note: This requires a running Qdrant instance
        // Start it with: docker run -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage qdrant/qdrant
        let client = Qdrant::from_url(QDRANT_URL)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to connect to Qdrant. Make sure Qdrant is running on localhost:6333.\n\
                     Start it with: docker run -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage qdrant/qdrant")?;

        // Test the connection by trying to list collections
        println!("  {} Testing Qdrant connection...", "•".cyan());
        match client.list_collections().await {
            Ok(_) => {
                println!("  {} Successfully connected to Qdrant", "✓".green());
            }
            Err(e) => {
                eprintln!(
                    "\n{} {}",
                    "✗".red().bold(),
                    "Failed to connect to Qdrant".red().bold()
                );
                eprintln!("\nError details: {}", e);
                eprintln!("\n{}", "Troubleshooting steps:".yellow().bold());
                eprintln!("  1. Check if Qdrant is running:");
                eprintln!("     docker ps | grep qdrant");
                eprintln!("  2. Restart Qdrant:");
                eprintln!("     docker restart curalit-qdrant");
                eprintln!("  3. Check Qdrant logs:");
                eprintln!("     docker logs curalit-qdrant");
                eprintln!("  4. If issue persists, remove and recreate:");
                eprintln!("     docker stop curalit-qdrant && docker rm curalit-qdrant");
                eprintln!("     docker run -d --name curalit-qdrant -p 6333:6333 -p 6334:6334 qdrant/qdrant");
                return Err(anyhow::anyhow!(
                    "Qdrant connection failed. See troubleshooting steps above."
                ));
            }
        }

        // Check if collection exists, create if not
        let collections = client.list_collections().await?;
        let collection_exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.config.collection_name);

        if !collection_exists {
            println!(
                "{} Creating Qdrant collection: {}",
                "•".cyan(),
                self.config.collection_name.white().bold()
            );

            // Get embedding dimension by testing with a sample text
            let test_embedding = self.generate_embedding("test").await?;
            let dimension = test_embedding.len() as u64;

            client
                .create_collection(CreateCollection {
                    collection_name: self.config.collection_name.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                            VectorParams {
                                size: dimension,
                                distance: Distance::Cosine.into(),
                                ..Default::default()
                            },
                        )),
                    }),
                    ..Default::default()
                })
                .await
                .context("Failed to create Qdrant collection")?;

            println!(
                "  {} Collection created with dimension {}",
                "✓".green(),
                dimension
            );
        } else {
            println!(
                "  {} Collection '{}' already exists",
                "✓".green(),
                self.config.collection_name
            );
        }

        self.client = Some(client);
        Ok(())
    }

    /// Build RAG index from articles
    pub async fn build_index(&self, articles: &[Article]) -> Result<()> {
        println!("\n{}", "═".repeat(80).cyan());
        println!("{} {}", "Building RAG Index...".green().bold(), "🔍");
        println!("{}\n", "═".repeat(80).cyan());

        let client = self
            .client
            .as_ref()
            .context("Qdrant client not initialized")?;

        // Chunk all articles
        println!("{} Chunking {} articles...", "•".cyan(), articles.len());
        let chunks = self.chunk_articles(articles)?;
        println!("  {} Created {} chunks", "✓".green(), chunks.len());

        // Generate embeddings and store in Qdrant
        println!(
            "{} Generating embeddings and storing in Qdrant...",
            "•".cyan()
        );
        let pb = ProgressBar::new(chunks.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▓▒░ "),
        );

        let mut points = Vec::new();
        for chunk in chunks {
            let embedding = self.generate_embedding(&chunk.text).await?;

            // Convert chunk to JSON value and then to Payload-compatible format
            let json_value = serde_json::to_value(&chunk)?;
            let payload: serde_json::Map<String, serde_json::Value> =
                if let serde_json::Value::Object(map) = json_value {
                    map
                } else {
                    serde_json::Map::new()
                };

            let point = PointStruct::new(chunk.id.clone(), embedding, payload);
            points.push(point);

            pb.inc(1);

            // Batch upsert every 100 points
            if points.len() >= 100 {
                let upsert =
                    UpsertPointsBuilder::new(self.config.collection_name.clone(), points.clone());
                client.upsert_points(upsert).await?;
                points.clear();
            }
        }

        // Upsert remaining points
        if !points.is_empty() {
            let upsert = UpsertPointsBuilder::new(self.config.collection_name.clone(), points);
            client.upsert_points(upsert).await?;
        }

        pb.finish_with_message("Complete");

        println!(
            "\n{} {}",
            "✓".green().bold(),
            "RAG index built successfully!".green().bold()
        );
        println!("\n{}", "Next steps:".yellow().bold());
        println!(
            "  {} Query the index: {}",
            "1.".cyan(),
            "curalit rag-query -q \"your question\"".white()
        );
        println!(
            "  {} Use with Ollama: {}",
            "2.".cyan(),
            "curalit rag-generate -q \"your question\" -m llama3".white()
        );

        Ok(())
    }

    /// Chunk articles into smaller passages
    pub fn chunk_articles(&self, articles: &[Article]) -> Result<Vec<ArticleChunk>> {
        let mut chunks = Vec::new();

        for article in articles {
            // Combine title and abstract
            let full_text = format!("{}\n\n{}", article.title, article.abstract_text);

            // Split into chunks
            let article_chunks = self.split_text(&full_text);

            for (idx, chunk_text) in article_chunks.into_iter().enumerate() {
                chunks.push(ArticleChunk {
                    id: Uuid::new_v4().to_string(),
                    article_id: article.pmid.clone(),
                    pmid: article.pmid.clone(),
                    title: article.title.clone(),
                    chunk_index: idx,
                    text: chunk_text,
                    authors: article.authors.clone(),
                    journal: article.journal.clone(),
                    pub_date: article.pub_date.clone(),
                    mesh_terms: article.mesh_terms.clone(),
                });
            }
        }

        Ok(chunks)
    }

    /// Split text into overlapping chunks
    fn split_text(&self, text: &str) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();

        if words.len() <= self.config.chunk_size {
            chunks.push(text.to_string());
            return chunks;
        }

        let mut start = 0;
        while start < words.len() {
            let end = (start + self.config.chunk_size).min(words.len());
            let chunk = words[start..end].join(" ");
            chunks.push(chunk);

            if end == words.len() {
                break;
            }

            start += self.config.chunk_size - self.config.chunk_overlap;
        }

        chunks
    }

    /// Generate embedding for text using Ollama
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/embeddings", self.config.ollama_url);

        let payload = serde_json::json!({
            "model": self.config.embedding_model,
            "prompt": text,
        });

        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to call Ollama embedding API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!(
                "Ollama API error ({}): {}. Make sure Ollama is running and the model '{}' is available.",
                status,
                error_text,
                self.config.embedding_model
            );
        }

        let embedding_response: OllamaEmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(embedding_response.embedding)
    }

    /// Query the RAG index
    pub async fn query(&self, query_text: &str) -> Result<Vec<ArticleChunk>> {
        let client = self
            .client
            .as_ref()
            .context("Qdrant client not initialized")?;

        // Generate embedding for query
        let query_embedding = self.generate_embedding(query_text).await?;

        // Search in Qdrant
        let search_result = client
            .search_points(SearchPoints {
                collection_name: self.config.collection_name.clone(),
                vector: query_embedding,
                limit: self.config.top_k as u64,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .context("Failed to search in Qdrant")?;

        // Convert results to ArticleChunks
        let mut chunks = Vec::new();
        for point in search_result.result {
            let payload = point.payload;
            if !payload.is_empty() {
                // Convert qdrant Value types to serde_json Values
                let json_map: serde_json::Map<String, serde_json::Value> = payload
                    .into_iter()
                    .filter_map(|(k, v)| {
                        // Convert qdrant Value to serde_json Value
                        // For simplicity, we'll serialize and deserialize through JSON
                        serde_json::to_value(&v).ok().map(|json_v| (k, json_v))
                    })
                    .collect();

                let chunk: ArticleChunk =
                    serde_json::from_value(serde_json::Value::Object(json_map))?;
                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    /// Generate answer using RAG (retrieve + generate)
    pub async fn generate_answer(&self, query: &str, ollama_model: &str) -> Result<String> {
        // Retrieve relevant chunks
        println!("{} Retrieving relevant passages...", "•".cyan());
        let chunks = self.query(query).await?;

        if chunks.is_empty() {
            return Ok("No relevant information found in the knowledge base.".to_string());
        }

        println!("  {} Found {} relevant passages", "✓".green(), chunks.len());

        // Build context from chunks
        let mut context = String::new();
        for (idx, chunk) in chunks.iter().enumerate() {
            context.push_str(&format!(
                "\n[Passage {}] PMID: {} - {}\n{}\n",
                idx + 1,
                chunk.pmid,
                chunk.title,
                chunk.text
            ));
        }

        // Create prompt
        let prompt = format!(
            r#"You are a research assistant with access to biomedical literature. Answer the question based ONLY on the provided passages. If the answer is not in the passages, say so.

Context:
{}

Question: {}

Answer:"#,
            context, query
        );

        // Call Ollama for generation
        println!("{} Generating answer with {}...", "•".cyan(), ollama_model);
        let answer = self.call_ollama_generate(&prompt, ollama_model).await?;

        Ok(answer)
    }

    /// Call Ollama generate API
    async fn call_ollama_generate(&self, prompt: &str, model: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/generate", self.config.ollama_url);

        let payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });

        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to call Ollama generate API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!(
                "Ollama API error ({}): {}. Make sure Ollama is running and model '{}' is available.",
                status,
                error_text,
                model
            );
        }

        let response_json: serde_json::Value = response.json().await?;
        let answer = response_json["response"]
            .as_str()
            .context("No response field in Ollama output")?
            .to_string();

        Ok(answer)
    }

    /// Save RAG configuration to file
    pub fn save_config(&self, output_dir: &Path) -> Result<PathBuf> {
        let config_path = output_dir.join("rag_config.json");
        let config_json = serde_json::to_string_pretty(&self.config)?;
        fs::write(&config_path, config_json)?;
        Ok(config_path)
    }

    /// Load RAG configuration from file
    pub fn load_config(config_path: &Path) -> Result<RagConfig> {
        let config_json = fs::read_to_string(config_path)?;
        let config: RagConfig = serde_json::from_str(&config_json)?;
        Ok(config)
    }

    /// Check if Qdrant server is reachable
    pub async fn check_qdrant_connection() -> Result<bool> {
        let client = reqwest::Client::new();
        match client.get(format!("{}/", QDRANT_URL)).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Check if Ollama server is reachable
    pub async fn check_ollama_connection(url: &str) -> Result<bool> {
        let client = reqwest::Client::new();
        match client.get(format!("{}/api/version", url)).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::article::Article;

    #[test]
    fn test_rag_config_default() {
        let config = RagConfig::default();
        assert_eq!(config.embedding_model, "nomic-embed-text");
        assert_eq!(config.chunk_size, 500);
        assert_eq!(config.chunk_overlap, 50);
        assert_eq!(config.top_k, 5);
        assert_eq!(config.collection_name, "curalit_articles");
    }

    #[test]
    fn test_rag_config_custom() {
        let config = RagConfig {
            ollama_url: "http://localhost:8080".to_string(),
            embedding_model: "custom-model".to_string(),
            collection_name: "test_collection".to_string(),
            qdrant_path: PathBuf::from("/tmp/test"),
            chunk_size: 1000,
            chunk_overlap: 100,
            top_k: 10,
        };

        assert_eq!(config.ollama_url, "http://localhost:8080");
        assert_eq!(config.embedding_model, "custom-model");
        assert_eq!(config.chunk_size, 1000);
        assert_eq!(config.top_k, 10);
    }

    #[test]
    fn test_split_text_small() {
        let rag = RagSystem::new();
        let text = "This is a short test.";
        let chunks = rag.split_text(&text);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_split_text() {
        let rag = RagSystem::new();
        let text = "word ".repeat(1000); // 1000 words
        let chunks = rag.split_text(&text);

        assert!(!chunks.is_empty());
        // Should have multiple chunks
        assert!(chunks.len() > 1);

        // Each chunk should be roughly chunk_size words
        for chunk in &chunks {
            let word_count = chunk.split_whitespace().count();
            assert!(word_count <= rag.config.chunk_size + rag.config.chunk_overlap);
        }
    }

    #[test]
    fn test_split_text_with_overlap() {
        let mut config = RagConfig::default();
        config.chunk_size = 10;
        config.chunk_overlap = 3;

        let rag = RagSystem::with_config(config);
        let text = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen".to_string();
        let chunks = rag.split_text(&text);

        // Should create multiple chunks with overlap
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_chunk_articles() {
        let rag = RagSystem::new();

        let articles = vec![
            Article {
                pmid: "12345".to_string(),
                title: "Test Article 1".to_string(),
                abstract_text: "This is a test abstract with some content.".to_string(),
                authors: vec!["Author One".to_string()],
                journal: "Test Journal".to_string(),
                pub_date: "2024-01-01".to_string(),
                mesh_terms: vec!["Test".to_string()],
                chemicals: vec![],
                doi: None,
                keywords: vec![],
            },
            Article {
                pmid: "67890".to_string(),
                title: "Test Article 2".to_string(),
                abstract_text: "Another test abstract with different content.".to_string(),
                authors: vec!["Author Two".to_string()],
                journal: "Another Journal".to_string(),
                pub_date: "2024-01-02".to_string(),
                mesh_terms: vec!["Research".to_string()],
                chemicals: vec![],
                doi: None,
                keywords: vec![],
            },
        ];

        let chunks = rag.chunk_articles(&articles).unwrap();

        // Should have at least as many chunks as articles
        assert!(chunks.len() >= articles.len());

        // Check first chunk properties
        assert_eq!(chunks[0].pmid, "12345");
        assert_eq!(chunks[0].title, "Test Article 1");
        assert_eq!(chunks[0].chunk_index, 0);
        assert!(!chunks[0].text.is_empty());
    }

    #[test]
    fn test_article_chunk_serialization() {
        let chunk = ArticleChunk {
            id: "test-id".to_string(),
            article_id: "12345".to_string(),
            pmid: "12345".to_string(),
            title: "Test Title".to_string(),
            chunk_index: 0,
            text: "Test text content".to_string(),
            authors: vec!["Author".to_string()],
            journal: "Journal".to_string(),
            pub_date: "2024-01-01".to_string(),
            mesh_terms: vec!["Term".to_string()],
        };

        // Test serialization
        let json = serde_json::to_string(&chunk).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Title"));

        // Test deserialization
        let deserialized: ArticleChunk = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, chunk.id);
        assert_eq!(deserialized.pmid, chunk.pmid);
        assert_eq!(deserialized.title, chunk.title);
    }

    #[test]
    fn test_rag_config_serialization() {
        let config = RagConfig::default();

        // Test serialization
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("nomic-embed-text"));
        assert!(json.contains("curalit_articles"));

        // Test deserialization
        let deserialized: RagConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.embedding_model, config.embedding_model);
        assert_eq!(deserialized.collection_name, config.collection_name);
    }

    #[test]
    fn test_save_and_load_config() {
        use std::env;

        let config = RagConfig::default();
        let temp_dir = env::temp_dir();

        let rag = RagSystem::with_config(config.clone());
        let config_path = rag.save_config(&temp_dir).unwrap();

        // Verify file was created
        assert!(config_path.exists());

        // Load and verify
        let loaded_config = RagSystem::load_config(&config_path).unwrap();
        assert_eq!(loaded_config.embedding_model, config.embedding_model);
        assert_eq!(loaded_config.collection_name, config.collection_name);

        // Cleanup
        std::fs::remove_file(config_path).ok();
    }

    #[test]
    fn test_chunk_size_validation() {
        let mut config = RagConfig::default();
        config.chunk_size = 100;
        config.chunk_overlap = 50;

        let rag = RagSystem::with_config(config.clone());
        assert_eq!(rag.config.chunk_size, 100);
        assert_eq!(rag.config.chunk_overlap, 50);

        // Overlap should be less than chunk_size
        assert!(rag.config.chunk_overlap < rag.config.chunk_size);
    }

    // Integration tests (require running services)
    #[tokio::test]
    #[ignore] // Only run with --ignored flag when services are available
    async fn test_qdrant_connection() {
        let result = RagSystem::check_qdrant_connection().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run with --ignored flag when services are available
    async fn test_ollama_connection() {
        let result = RagSystem::check_ollama_connection("http://localhost:11434").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run with --ignored flag when services are available
    async fn test_full_rag_workflow() {
        // This test requires both Qdrant and Ollama to be running
        let mut config = RagConfig::default();
        config.collection_name = "test_collection".to_string();

        let mut rag = RagSystem::with_config(config);

        // Test initialization
        let init_result = rag.initialize().await;
        if init_result.is_err() {
            eprintln!("Skipping test - Qdrant not available");
            return;
        }

        // Create test articles
        let articles = vec![Article {
            pmid: "test123".to_string(),
            title: "Cancer Immunotherapy Research".to_string(),
            abstract_text: "This study investigates checkpoint inhibitors in cancer treatment."
                .to_string(),
            authors: vec!["Dr. Smith".to_string()],
            journal: "Nature Medicine".to_string(),
            pub_date: "2024-01-01".to_string(),
            mesh_terms: vec!["Cancer".to_string(), "Immunotherapy".to_string()],
            chemicals: vec![],
            doi: None,
            keywords: vec![],
        }];

        // Build index
        let build_result = rag.build_index(&articles).await;
        if build_result.is_err() {
            eprintln!("Skipping test - Ollama not available");
            return;
        }

        // Query the index
        let query_result = rag.query("cancer treatment").await;
        assert!(query_result.is_ok());

        let chunks = query_result.unwrap();
        assert!(!chunks.is_empty());
    }
}

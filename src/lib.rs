//! CuraLit - Literature-Driven LLM Generator
//!
//! A tool for extracting relevant articles from PubMed XML datasets and
//! generating custom LLM models for research purposes.
//!
//! # Features
//!
//! * Memory-efficient streaming XML parser for large datasets
//! * Configurable keyword matching (AND/OR logic)
//! * Checkpoint system for resumable operations
//! * Statistical analysis with threshold warnings
//! * Automatic Python visualization generation
//! * Ollama Modelfile generation for fine-tuning
//! * RAG (Retrieval-Augmented Generation) for accurate fact retrieval
//!
//! # Example Usage
//!
//! ```bash
//! # Search for articles
//! curalit search -k "cancer" -k "immunotherapy" -d ./data -o results
//!
//! # Generate statistics
//! curalit stats -c results.csv
//!
//! # Generate Ollama model
//! curalit generate -c results.csv -m my-medical-llm -b llama3
//! ```

pub mod article;
pub mod checkpoint;
pub mod cli;
pub mod database;
pub mod modelfile;
pub mod parser;
pub mod rag;
pub mod runner;
pub mod statistics;
pub mod visualizer;

// Re-export commonly used types
pub use article::Article;
pub use checkpoint::CheckpointManager;
pub use cli::{Cli, Commands, KeywordLogic};
pub use database::{ArticleDatabase, ArticleInfo, DatabaseStats};
pub use modelfile::ModelfileGenerator;
pub use parser::PubMedParser;
pub use rag::{ArticleChunk, RagConfig, RagSystem};
pub use runner::CuraLitRunner;
pub use statistics::{Statistics, StatisticsAnalyzer};
pub use visualizer::VisualizationGenerator;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_imports() {
        // Ensure all modules are accessible
        let _ = Article::new("test".to_string());
    }
}

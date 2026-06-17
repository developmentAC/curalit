//! Integration tests for RAG functionality
//!
//! These tests require:
//! - Qdrant running on localhost:6333
//! - Ollama running on localhost:11434 with nomic-embed-text model
//!
//! Run with: cargo test --test rag_integration_test -- --ignored
//! Or: cargo test --test rag_integration_test (will skip integration tests)

use anyhow::Result;
use curalit::article::Article;
use curalit::rag::{RagConfig, RagSystem, QDRANT_URL};
use std::path::PathBuf;

/// Helper to check if Qdrant is available
async fn is_qdrant_available() -> bool {
    RagSystem::check_qdrant_connection().await.unwrap_or(false)
}

/// Helper to check if Ollama is available
async fn is_ollama_available() -> bool {
    RagSystem::check_ollama_connection("http://localhost:11434")
        .await
        .unwrap_or(false)
}

/// Create test articles for integration tests
fn create_test_articles() -> Vec<Article> {
    vec![
        Article {
            pmid: "test001".to_string(),
            title: "CAR-T Cell Therapy for Cancer Treatment".to_string(),
            abstract_text: "Chimeric antigen receptor T-cell therapy represents a revolutionary approach to cancer treatment. This study examines the efficacy of CAR-T therapy in treating various hematologic malignancies including leukemia and lymphoma.".to_string(),
            authors: vec!["Smith, J.".to_string(), "Johnson, A.".to_string()],
            journal: "Nature Medicine".to_string(),
            pub_date: "2024-01-15".to_string(),
            mesh_terms: vec!["CAR-T Therapy".to_string(), "Cancer".to_string(), "Immunotherapy".to_string()],
            chemicals: vec![],
            doi: None,
            keywords: vec![],
        },
        Article {
            pmid: "test002".to_string(),
            title: "Checkpoint Inhibitors in Melanoma".to_string(),
            abstract_text: "Immune checkpoint inhibitors such as pembrolizumab and nivolumab have transformed melanoma treatment. This meta-analysis reviews clinical trial data demonstrating improved survival rates.".to_string(),
            authors: vec!["Williams, R.".to_string(), "Brown, T.".to_string()],
            journal: "Journal of Clinical Oncology".to_string(),
            pub_date: "2024-02-20".to_string(),
            mesh_terms: vec!["Checkpoint Inhibitors".to_string(), "Melanoma".to_string(), "PD-1".to_string()],
            chemicals: vec!["Pembrolizumab".to_string(), "Nivolumab".to_string()],
            doi: None,
            keywords: vec![],
        },
        Article {
            pmid: "test003".to_string(),
            title: "Mechanisms of Drug Resistance in Cancer".to_string(),
            abstract_text: "Understanding resistance mechanisms is crucial for improving cancer therapy. This review explores genetic mutations, epigenetic changes, and tumor microenvironment factors contributing to treatment resistance.".to_string(),
            authors: vec!["Davis, M.".to_string()],
            journal: "Cancer Research".to_string(),
            pub_date: "2024-03-10".to_string(),
            mesh_terms: vec!["Drug Resistance".to_string(), "Cancer".to_string(), "Tumor Microenvironment".to_string()],
            chemicals: vec![],
            doi: None,
            keywords: vec![],
        },
    ]
}

#[tokio::test]
async fn test_rag_config_creation() {
    let config = RagConfig::default();
    assert_eq!(config.ollama_url, "http://localhost:11434");
    assert_eq!(config.embedding_model, "nomic-embed-text");
    assert_eq!(config.collection_name, "curalit_articles");
}

#[tokio::test]
async fn test_rag_system_initialization() {
    let rag = RagSystem::new();
    assert_eq!(rag.config.chunk_size, 500);
    assert_eq!(rag.config.top_k, 5);
}

#[tokio::test]
async fn test_text_chunking() {
    let rag = RagSystem::new();
    let articles = create_test_articles();

    let chunks = rag.chunk_articles(&articles).unwrap();

    // Should have at least one chunk per article
    assert!(chunks.len() >= articles.len());

    // Verify chunk properties
    for chunk in &chunks {
        assert!(!chunk.id.is_empty());
        assert!(!chunk.pmid.is_empty());
        assert!(!chunk.title.is_empty());
        assert!(!chunk.text.is_empty());
    }

    // First chunk should be from first article
    assert_eq!(chunks[0].pmid, "test001");
    assert!(chunks[0].text.contains("CAR-T"));
}

#[tokio::test]
async fn test_config_save_load() {
    let temp_dir = std::env::temp_dir().join("curalit_test_rag");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let config = RagConfig {
        ollama_url: "http://test:8080".to_string(),
        embedding_model: "test-model".to_string(),
        collection_name: "test_collection".to_string(),
        qdrant_path: PathBuf::from("/tmp/test"),
        chunk_size: 300,
        chunk_overlap: 30,
        top_k: 3,
    };

    let rag = RagSystem::with_config(config.clone());
    let config_path = rag.save_config(&temp_dir).unwrap();

    assert!(config_path.exists());

    let loaded = RagSystem::load_config(&config_path).unwrap();
    assert_eq!(loaded.ollama_url, config.ollama_url);
    assert_eq!(loaded.embedding_model, config.embedding_model);
    assert_eq!(loaded.chunk_size, config.chunk_size);

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[tokio::test]
#[ignore] // Run with --ignored when Qdrant is available
async fn test_qdrant_connectivity() {
    let available = is_qdrant_available().await;

    if !available {
        eprintln!("⚠ Qdrant not available at {}. Start with:", QDRANT_URL);
        eprintln!("  docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant");
        return;
    }

    println!("✓ Qdrant is available at {}", QDRANT_URL);
    assert!(available);
}

#[tokio::test]
#[ignore] // Run with --ignored when Ollama is available
async fn test_ollama_connectivity() {
    let available = is_ollama_available().await;

    if !available {
        eprintln!("⚠ Ollama not available. Make sure it's running and:");
        eprintln!("  ollama pull nomic-embed-text");
        return;
    }

    println!("✓ Ollama is available");
    assert!(available);
}

#[tokio::test]
#[ignore] // Run with --ignored when services are available
async fn test_rag_initialization_with_server() -> Result<()> {
    if !is_qdrant_available().await {
        eprintln!("Skipping test: Qdrant not available");
        return Ok(());
    }

    let mut config = RagConfig::default();
    config.collection_name = format!("test_collection_{}", uuid::Uuid::new_v4());

    let mut rag = RagSystem::with_config(config);

    // This should succeed if Qdrant is running
    let result = rag.initialize().await;

    if let Err(e) = result {
        eprintln!("Failed to initialize RAG: {}", e);
        return Err(e);
    }

    println!("✓ RAG system initialized successfully");
    Ok(())
}

#[tokio::test]
#[ignore] // Run with --ignored when services are available
async fn test_rag_build_index() -> Result<()> {
    if !is_qdrant_available().await || !is_ollama_available().await {
        eprintln!("Skipping test: Required services not available");
        return Ok(());
    }

    let mut config = RagConfig::default();
    config.collection_name = format!("test_build_{}", uuid::Uuid::new_v4());

    let mut rag = RagSystem::with_config(config);
    rag.initialize().await?;

    let articles = create_test_articles();

    // Build the index
    let result = rag.build_index(&articles).await;

    if let Err(e) = result {
        eprintln!("Failed to build index: {}", e);
        return Err(e);
    }

    println!(
        "✓ RAG index built successfully with {} articles",
        articles.len()
    );
    Ok(())
}

#[tokio::test]
#[ignore] // Run with --ignored when services are available
async fn test_rag_query_retrieval() -> Result<()> {
    if !is_qdrant_available().await || !is_ollama_available().await {
        eprintln!("Skipping test: Required services not available");
        return Ok(());
    }

    let mut config = RagConfig::default();
    config.collection_name = format!("test_query_{}", uuid::Uuid::new_v4());
    config.top_k = 3;

    let mut rag = RagSystem::with_config(config);
    rag.initialize().await?;

    // Build index
    let articles = create_test_articles();
    rag.build_index(&articles).await?;

    // Query for CAR-T therapy
    let results = rag.query("CAR-T cell therapy cancer treatment").await?;

    assert!(!results.is_empty(), "Should retrieve relevant passages");
    assert!(results.len() <= 3, "Should not exceed top_k");

    // First result should be most relevant (about CAR-T)
    println!("✓ Retrieved {} relevant passages", results.len());
    println!("  Top result: {}", results[0].title);

    Ok(())
}

#[tokio::test]
#[ignore] // Run with --ignored when services are available
async fn test_rag_generate_answer() -> Result<()> {
    if !is_qdrant_available().await || !is_ollama_available().await {
        eprintln!("Skipping test: Required services not available");
        return Ok(());
    }

    // Check if llama3 is available
    let client = reqwest::Client::new();
    let models_resp = client.get("http://localhost:11434/api/tags").send().await?;

    if !models_resp.status().is_success() {
        eprintln!("Skipping test: Cannot check available models");
        return Ok(());
    }

    let mut config = RagConfig::default();
    config.collection_name = format!("test_generate_{}", uuid::Uuid::new_v4());

    let mut rag = RagSystem::with_config(config);
    rag.initialize().await?;

    // Build index
    let articles = create_test_articles();
    rag.build_index(&articles).await?;

    // Generate answer
    let question = "What is CAR-T therapy?";
    let answer = rag.generate_answer(question, "llama3").await?;

    assert!(!answer.is_empty(), "Should generate an answer");
    println!("✓ Generated answer for: {}", question);
    println!("  Answer length: {} chars", answer.len());

    Ok(())
}

#[tokio::test]
#[ignore] // Run with --ignored when services are available
async fn test_rag_multiple_queries() -> Result<()> {
    if !is_qdrant_available().await || !is_ollama_available().await {
        eprintln!("Skipping test: Required services not available");
        return Ok(());
    }

    let mut config = RagConfig::default();
    config.collection_name = format!("test_multi_{}", uuid::Uuid::new_v4());

    let mut rag = RagSystem::with_config(config);
    rag.initialize().await?;

    let articles = create_test_articles();
    rag.build_index(&articles).await?;

    // Test multiple different queries
    let queries = vec![
        "checkpoint inhibitors melanoma",
        "drug resistance mechanisms",
        "immunotherapy cancer treatment",
    ];

    for query in queries {
        let results = rag.query(query).await?;
        assert!(
            !results.is_empty(),
            "Query '{}' should return results",
            query
        );
        println!("✓ Query '{}': {} results", query, results.len());
    }

    Ok(())
}

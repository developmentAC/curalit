//! Tests for Modelfile generation and training data creation

use anyhow::Result;
use curalit::article::Article;
use curalit::modelfile::ModelfileGenerator;
use std::fs;
use std::path::PathBuf;

/// Helper to create test articles
fn create_test_articles() -> Vec<Article> {
    vec![
        Article {
            pmid: "10001".to_string(),
            title: "First Cancer Research Article".to_string(),
            authors: vec!["Smith, J.".to_string(), "Doe, A.".to_string()],
            abstract_text:
                "This is the first test article about cancer research and immunotherapy."
                    .to_string(),
            journal: "Nature".to_string(),
            pub_date: "2024-01-01".to_string(),
            mesh_terms: vec!["Neoplasms".to_string(), "Immunotherapy".to_string()],
            chemicals: vec!["PD-1".to_string()],
            doi: Some("10.1038/test.001".to_string()),
            keywords: vec!["cancer".to_string(), "immunotherapy".to_string()],
        },
        Article {
            pmid: "10002".to_string(),
            title: "Second Cancer Treatment Study".to_string(),
            authors: vec!["Johnson, B.".to_string()],
            abstract_text:
                "This is the second test article about novel cancer treatment approaches."
                    .to_string(),
            journal: "Science".to_string(),
            pub_date: "2024-02-01".to_string(),
            mesh_terms: vec!["Neoplasms".to_string(), "Drug Therapy".to_string()],
            chemicals: vec!["Chemotherapy Agent".to_string()],
            doi: None,
            keywords: vec!["cancer".to_string(), "treatment".to_string()],
        },
        Article {
            pmid: "10003".to_string(),
            title: "Third Melanoma Research".to_string(),
            authors: vec![
                "Williams, C.".to_string(),
                "Brown, D.".to_string(),
                "Davis, E.".to_string(),
            ],
            abstract_text:
                "This article focuses on melanoma treatment using checkpoint inhibitors."
                    .to_string(),
            journal: "Cell".to_string(),
            pub_date: "2024-03-01".to_string(),
            mesh_terms: vec!["Melanoma".to_string(), "Checkpoint Inhibitors".to_string()],
            chemicals: vec!["Pembrolizumab".to_string(), "Nivolumab".to_string()],
            doi: Some("10.1016/test.003".to_string()),
            keywords: vec!["melanoma".to_string(), "checkpoint".to_string()],
        },
    ]
}

/// Test basic modelfile generation
#[test]
fn test_modelfile_generation() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_modelfile_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new("test-model".to_string(), "llama3".to_string());
    let articles = create_test_articles();
    let timestamp = "15Jun2024_120000";

    generator.generate(&articles, "results", &temp_dir, timestamp)?;

    // Check that files were created
    let modelfile_path = temp_dir.join("Modelfile_test-model_15Jun2024_120000");
    let training_path = temp_dir.join("results_15Jun2024_120000_training.jsonl");
    let prompt_path = temp_dir.join("results_15Jun2024_120000_system_prompt.txt");

    assert!(modelfile_path.exists(), "Modelfile should be created");
    assert!(training_path.exists(), "Training data should be created");
    assert!(prompt_path.exists(), "System prompt should be created");

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test training data format (JSONL)
#[test]
fn test_training_data_format() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_training_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new("test-model".to_string(), "llama3".to_string());
    let articles = create_test_articles();
    let timestamp = "15Jun2024_120000";

    generator.generate(&articles, "results", &temp_dir, timestamp)?;

    // Read training data
    let training_path = temp_dir.join("results_15Jun2024_120000_training.jsonl");
    let content = fs::read_to_string(&training_path)?;

    // Should have one JSON object per line
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), articles.len());

    // Each line should be valid JSON
    for line in lines {
        let json: serde_json::Value = serde_json::from_str(line)?;
        assert!(json.is_object());
        assert!(json.get("id").is_some());
        assert!(json.get("text").is_some());
    }

    // Check first entry
    let first_line = content.lines().next().unwrap();
    let first_json: serde_json::Value = serde_json::from_str(first_line)?;
    assert_eq!(first_json["id"], "10001");

    let text = first_json["text"].as_str().unwrap();
    assert!(text.contains("First Cancer Research Article"));
    assert!(text.contains("Smith, J."));
    assert!(text.contains("immunotherapy"));

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test Modelfile content structure
#[test]
fn test_modelfile_content() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_modelfile_content_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new("cancer-research".to_string(), "llama3".to_string());
    let articles = create_test_articles();
    let timestamp = "15Jun2024_120000";

    generator.generate(&articles, "results", &temp_dir, timestamp)?;

    // Read Modelfile
    let modelfile_path = temp_dir.join("Modelfile_cancer-research_15Jun2024_120000");
    let content = fs::read_to_string(&modelfile_path)?;

    // Should contain expected sections
    assert!(content.contains("FROM llama3"));
    assert!(content.contains("SYSTEM"));
    assert!(content.contains("research assistant"));
    assert!(content.contains("biomedical") || content.contains("literature"));

    // Should reference the model name
    assert!(content.contains("cancer-research") || content.contains("Model:"));

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test system prompt content
#[test]
fn test_system_prompt_content() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_prompt_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new("test-model".to_string(), "llama3".to_string());
    let articles = create_test_articles();
    let timestamp = "15Jun2024_120000";

    generator.generate(&articles, "results", &temp_dir, timestamp)?;

    // Read system prompt
    let prompt_path = temp_dir.join("results_15Jun2024_120000_system_prompt.txt");
    let content = fs::read_to_string(&prompt_path)?;

    // Should contain relevant information
    assert!(!content.is_empty());
    assert!(content.len() > 50); // Should have substantial content

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test generation with empty article list
#[test]
fn test_generation_with_empty_articles() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_empty_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new("test-model".to_string(), "llama3".to_string());
    let articles: Vec<Article> = vec![];
    let timestamp = "15Jun2024_120000";

    let result = generator.generate(&articles, "results", &temp_dir, timestamp);

    // Should still create files (even if empty/minimal)
    assert!(result.is_ok());

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test generation with single article
#[test]
fn test_generation_with_single_article() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_single_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new("test-model".to_string(), "llama3".to_string());
    let articles = vec![create_test_articles()[0].clone()];
    let timestamp = "15Jun2024_120000";

    generator.generate(&articles, "results", &temp_dir, timestamp)?;

    // Check training data has exactly one entry
    let training_path = temp_dir.join("results_15Jun2024_120000_training.jsonl");
    let content = fs::read_to_string(&training_path)?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1);

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test generation with large article set
#[test]
fn test_generation_with_many_articles() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_many_test");
    fs::create_dir_all(&temp_dir)?;

    // Create 100 articles
    let mut articles = Vec::new();
    for i in 1..=100 {
        articles.push(Article {
            pmid: format!("{:08}", i),
            title: format!("Article {}", i),
            authors: vec![format!("Author {}", i)],
            abstract_text: format!("Abstract for article {}", i),
            journal: "Test Journal".to_string(),
            pub_date: "2024-01-01".to_string(),
            mesh_terms: vec![],
            chemicals: vec![],
            doi: None,
            keywords: vec![],
        });
    }

    let generator = ModelfileGenerator::new("large-model".to_string(), "llama3".to_string());
    let timestamp = "15Jun2024_120000";

    generator.generate(&articles, "results", &temp_dir, timestamp)?;

    // Check training data has 100 entries
    let training_path = temp_dir.join("results_15Jun2024_120000_training.jsonl");
    let content = fs::read_to_string(&training_path)?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 100);

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test different base models
#[test]
fn test_different_base_models() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_base_models_test");
    fs::create_dir_all(&temp_dir)?;

    let base_models = vec!["llama3", "mistral", "gemma", "llama2"];
    let articles = create_test_articles();

    for (idx, base_model) in base_models.iter().enumerate() {
        let generator =
            ModelfileGenerator::new(format!("test-model-{}", idx), base_model.to_string());
        let timestamp = format!("15Jun2024_12000{}", idx);

        generator.generate(&articles, "results", &temp_dir, &timestamp)?;

        // Check Modelfile references correct base model
        let modelfile_path = temp_dir.join(format!("Modelfile_test-model-{}_{}", idx, timestamp));
        let content = fs::read_to_string(&modelfile_path)?;
        assert!(content.contains(&format!("FROM {}", base_model)));
    }

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test special characters in model names
#[test]
fn test_special_characters_in_model_name() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_special_name_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new(
        "my-cancer-research-llm-v2".to_string(),
        "llama3".to_string(),
    );
    let articles = create_test_articles();
    let timestamp = "15Jun2024_120000";

    let result = generator.generate(&articles, "results", &temp_dir, timestamp);
    assert!(result.is_ok());

    // Check files were created with proper naming
    let modelfile_path = temp_dir.join("Modelfile_my-cancer-research-llm-v2_15Jun2024_120000");
    assert!(modelfile_path.exists());

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test that all article fields are included in training data
#[test]
fn test_complete_article_data_in_training() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_complete_data_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new("test-model".to_string(), "llama3".to_string());
    let articles = create_test_articles();
    let timestamp = "15Jun2024_120000";

    generator.generate(&articles, "results", &temp_dir, timestamp)?;

    // Read and parse training data
    let training_path = temp_dir.join("results_15Jun2024_120000_training.jsonl");
    let content = fs::read_to_string(&training_path)?;
    let first_line = content.lines().next().unwrap();
    let json: serde_json::Value = serde_json::from_str(first_line)?;

    let text = json["text"].as_str().unwrap().to_lowercase();

    // Check all major fields are present
    assert!(text.contains("first cancer research article")); // Title
    assert!(text.contains("smith")); // Authors
    assert!(text.contains("immunotherapy")); // Abstract
    assert!(text.contains("nature")); // Journal
    assert!(text.contains("neoplasms") || text.contains("mesh")); // MeSH terms

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test file naming conventions
#[test]
fn test_file_naming_conventions() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_naming_test");
    fs::create_dir_all(&temp_dir)?;

    let generator = ModelfileGenerator::new("my-model".to_string(), "llama3".to_string());
    let articles = create_test_articles();
    let timestamp = "15Jun2024_120000";
    let prefix = "test_results";

    generator.generate(&articles, prefix, &temp_dir, timestamp)?;

    // Check all expected files exist with correct naming
    let modelfile = temp_dir.join("Modelfile_my-model_15Jun2024_120000");
    let training = temp_dir.join("test_results_15Jun2024_120000_training.jsonl");
    let prompt = temp_dir.join("test_results_15Jun2024_120000_system_prompt.txt");

    assert!(modelfile.exists(), "Modelfile naming incorrect");
    assert!(training.exists(), "Training file naming incorrect");
    assert!(prompt.exists(), "Prompt file naming incorrect");

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

//! Comprehensive tests for checkpoint and resume functionality

use anyhow::Result;
use curalit::article::Article;
use curalit::checkpoint::CheckpointManager;
use std::fs;
use std::path::PathBuf;

/// Helper to create test articles
fn create_test_articles() -> Vec<Article> {
    vec![
        Article {
            pmid: "11111111".to_string(),
            title: "First Article".to_string(),
            authors: vec!["Author One".to_string()],
            abstract_text: "First article abstract.".to_string(),
            journal: "Journal A".to_string(),
            pub_date: "2024-01-01".to_string(),
            mesh_terms: vec!["Term A".to_string()],
            chemicals: vec![],
            doi: Some("10.1000/a".to_string()),
            keywords: vec!["keyword1".to_string()],
        },
        Article {
            pmid: "22222222".to_string(),
            title: "Second Article".to_string(),
            authors: vec!["Author Two".to_string(), "Author Three".to_string()],
            abstract_text: "Second article abstract.".to_string(),
            journal: "Journal B".to_string(),
            pub_date: "2024-02-01".to_string(),
            mesh_terms: vec!["Term B".to_string()],
            chemicals: vec!["Chemical B".to_string()],
            doi: None,
            keywords: vec!["keyword2".to_string()],
        },
        Article {
            pmid: "33333333".to_string(),
            title: "Third Article".to_string(),
            authors: vec!["Author Four".to_string()],
            abstract_text: "Third article abstract.".to_string(),
            journal: "Journal C".to_string(),
            pub_date: "2024-03-01".to_string(),
            mesh_terms: vec!["Term C".to_string(), "Term D".to_string()],
            chemicals: vec![],
            doi: Some("10.1000/c".to_string()),
            keywords: vec!["keyword3".to_string()],
        },
    ]
}

/// Test creating a new checkpoint
#[test]
fn test_checkpoint_creation() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_checkpoint_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;

    // File should exist
    assert!(checkpoint_path.exists());

    // Should have zero articles initially
    assert_eq!(checkpoint.article_count(), 0);

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test adding articles to checkpoint
#[test]
fn test_add_articles_to_checkpoint() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_add_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;

    let articles = create_test_articles();

    // Add each article
    for article in &articles {
        checkpoint.add_article(article)?;
    }

    // Check count
    assert_eq!(checkpoint.article_count(), 3);

    // Finalize
    checkpoint.finalize()?;

    // File should exist and have content
    let content = fs::read_to_string(&checkpoint_path)?;
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 4); // Header + 3 articles

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test loading articles from checkpoint
#[test]
fn test_load_articles_from_checkpoint() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_load_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");

    // Create checkpoint and add articles
    {
        let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;
        let articles = create_test_articles();
        for article in &articles {
            checkpoint.add_article(article)?;
        }
        checkpoint.finalize()?;
    }

    // Load checkpoint
    let checkpoint = CheckpointManager::load(&checkpoint_path)?;
    let loaded_articles = checkpoint.load_articles()?;

    // Verify loaded articles
    assert_eq!(loaded_articles.len(), 3);
    assert_eq!(loaded_articles[0].pmid, "11111111");
    assert_eq!(loaded_articles[1].pmid, "22222222");
    assert_eq!(loaded_articles[2].pmid, "33333333");

    assert_eq!(loaded_articles[0].title, "First Article");
    assert_eq!(loaded_articles[1].title, "Second Article");
    assert_eq!(loaded_articles[2].title, "Third Article");

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test resuming from checkpoint
#[test]
fn test_resume_from_checkpoint() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_resume_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let articles = create_test_articles();

    // Create initial checkpoint with first 2 articles
    {
        let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;
        checkpoint.add_article(&articles[0])?;
        checkpoint.add_article(&articles[1])?;
        checkpoint.finalize()?;
    }

    // Verify initial state
    let content = fs::read_to_string(&checkpoint_path)?;
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 3); // Header + 2 articles

    // Resume and add more articles
    {
        let checkpoint = CheckpointManager::new(&checkpoint_path, true)?;
        assert_eq!(checkpoint.article_count(), 2); // Should count existing
        checkpoint.add_article(&articles[2])?;
        checkpoint.finalize()?;
    }

    // Verify resumed state
    let final_content = fs::read_to_string(&checkpoint_path)?;
    let final_lines: Vec<&str> = final_content.lines().collect();
    assert_eq!(final_lines.len(), 4); // Header + 3 articles

    // Load and verify all articles
    let checkpoint = CheckpointManager::load(&checkpoint_path)?;
    let loaded = checkpoint.load_articles()?;
    assert_eq!(loaded.len(), 3);

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test checkpoint with CSV header
#[test]
fn test_checkpoint_header() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_header_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;
    checkpoint.finalize()?;

    // Read file and check header
    let content = fs::read_to_string(&checkpoint_path)?;
    let lines: Vec<&str> = content.lines().collect();

    assert!(lines.len() >= 1);
    let header = lines[0];

    // Should contain expected column names
    assert!(header.contains("PMID"));
    assert!(header.contains("Title"));
    assert!(header.contains("Authors"));
    assert!(header.contains("Abstract"));
    assert!(header.contains("Journal"));

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test checkpoint data integrity (round-trip)
#[test]
fn test_checkpoint_data_integrity() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_integrity_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let original_articles = create_test_articles();

    // Save articles
    {
        let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;
        for article in &original_articles {
            checkpoint.add_article(article)?;
        }
        checkpoint.finalize()?;
    }

    // Load articles
    let checkpoint = CheckpointManager::load(&checkpoint_path)?;
    let loaded_articles = checkpoint.load_articles()?;

    // Compare each article
    assert_eq!(loaded_articles.len(), original_articles.len());

    for (original, loaded) in original_articles.iter().zip(loaded_articles.iter()) {
        assert_eq!(loaded.pmid, original.pmid);
        assert_eq!(loaded.title, original.title);
        assert_eq!(loaded.authors, original.authors);
        assert_eq!(loaded.abstract_text, original.abstract_text);
        assert_eq!(loaded.journal, original.journal);
        assert_eq!(loaded.pub_date, original.pub_date);
        assert_eq!(loaded.mesh_terms, original.mesh_terms);
        assert_eq!(loaded.chemicals, original.chemicals);
        assert_eq!(loaded.doi, original.doi);
        assert_eq!(loaded.keywords, original.keywords);
    }

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test checkpoint with special characters
#[test]
fn test_checkpoint_special_characters() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_special_chars_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");

    let article = Article {
        pmid: "99999999".to_string(),
        title: "Article with \"quotes\" and, commas".to_string(),
        authors: vec!["O'Brien, John".to_string()],
        abstract_text: "Text with newline\nand tab\tcharacters.".to_string(),
        journal: "Journal; with semicolons".to_string(),
        pub_date: "2024-01-01".to_string(),
        mesh_terms: vec![],
        chemicals: vec![],
        doi: None,
        keywords: vec![],
    };

    // Save article
    {
        let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;
        checkpoint.add_article(&article)?;
        checkpoint.finalize()?;
    }

    // Load and verify
    let checkpoint = CheckpointManager::load(&checkpoint_path)?;
    let loaded = checkpoint.load_articles()?;

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].pmid, article.pmid);
    assert_eq!(loaded[0].title, article.title);
    assert_eq!(loaded[0].authors, article.authors);

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test multiple resume operations
#[test]
fn test_multiple_resumes() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_multiple_resume_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let articles = create_test_articles();

    // First session: add 1 article
    {
        let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;
        checkpoint.add_article(&articles[0])?;
        checkpoint.finalize()?;
    }

    // Second session: resume and add 1 article
    {
        let checkpoint = CheckpointManager::new(&checkpoint_path, true)?;
        assert_eq!(checkpoint.article_count(), 1);
        checkpoint.add_article(&articles[1])?;
        checkpoint.finalize()?;
    }

    // Third session: resume and add 1 article
    {
        let checkpoint = CheckpointManager::new(&checkpoint_path, true)?;
        assert_eq!(checkpoint.article_count(), 2);
        checkpoint.add_article(&articles[2])?;
        checkpoint.finalize()?;
    }

    // Final verification
    let checkpoint = CheckpointManager::load(&checkpoint_path)?;
    let loaded = checkpoint.load_articles()?;
    assert_eq!(loaded.len(), 3);

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test checkpoint count accuracy
#[test]
fn test_checkpoint_count_accuracy() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_count_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;

    let articles = create_test_articles();

    assert_eq!(checkpoint.article_count(), 0);

    checkpoint.add_article(&articles[0])?;
    assert_eq!(checkpoint.article_count(), 1);

    checkpoint.add_article(&articles[1])?;
    assert_eq!(checkpoint.article_count(), 2);

    checkpoint.add_article(&articles[2])?;
    assert_eq!(checkpoint.article_count(), 3);

    checkpoint.finalize()?;

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test loading non-existent checkpoint
#[test]
fn test_load_nonexistent_checkpoint() {
    let checkpoint_path = PathBuf::from("/nonexistent/path/checkpoint.csv");
    let result = CheckpointManager::load(&checkpoint_path);

    assert!(result.is_err());
}

/// Test checkpoint with empty article
#[test]
fn test_checkpoint_empty_article() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_empty_article_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;

    let empty_article = Article::new("00000000".to_string());
    checkpoint.add_article(&empty_article)?;
    checkpoint.finalize()?;

    // Load and verify
    let checkpoint = CheckpointManager::load(&checkpoint_path)?;
    let loaded = checkpoint.load_articles()?;

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].pmid, "00000000");

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test checkpoint file path retrieval
#[test]
fn test_checkpoint_file_path() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_path_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;

    let retrieved_path = checkpoint.file_path();
    assert_eq!(retrieved_path, checkpoint_path);

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Test large checkpoint (performance)
#[test]
fn test_large_checkpoint() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("curalit_large_checkpoint_test");
    fs::create_dir_all(&temp_dir)?;

    let checkpoint_path = temp_dir.join("test_checkpoint.csv");
    let checkpoint = CheckpointManager::new(&checkpoint_path, false)?;

    // Add 1000 articles
    for i in 0..1000 {
        let article = Article {
            pmid: format!("{:08}", i),
            title: format!("Article {}", i),
            authors: vec![format!("Author {}", i)],
            abstract_text: format!("Abstract {}", i),
            journal: "Test Journal".to_string(),
            pub_date: "2024-01-01".to_string(),
            mesh_terms: vec![],
            chemicals: vec![],
            doi: None,
            keywords: vec![],
        };
        checkpoint.add_article(&article)?;
    }

    checkpoint.finalize()?;

    // Verify count
    assert_eq!(checkpoint.article_count(), 1000);

    // Load and verify
    let checkpoint = CheckpointManager::load(&checkpoint_path)?;
    let loaded = checkpoint.load_articles()?;
    assert_eq!(loaded.len(), 1000);

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

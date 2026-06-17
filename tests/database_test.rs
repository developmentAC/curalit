//! Integration tests for the database module

use anyhow::Result;
use curalit::article::Article;
use curalit::database::ArticleDatabase;
use std::fs;
use std::path::PathBuf;

/// Helper function to create test articles
fn create_test_articles() -> Vec<Article> {
    vec![
        Article {
            pmid: "12345678".to_string(),
            title: "Cancer Immunotherapy Mechanisms".to_string(),
            authors: vec!["Smith, J.".to_string(), "Johnson, A.".to_string()],
            abstract_text: "This study explores novel checkpoint inhibitors in cancer treatment."
                .to_string(),
            journal: "Nature Medicine".to_string(),
            pub_date: "2024-03-15".to_string(),
            mesh_terms: vec!["Neoplasms".to_string(), "Immunotherapy".to_string()],
            chemicals: vec!["PD-1".to_string()],
            doi: Some("10.1038/nm.1234".to_string()),
            keywords: vec!["cancer".to_string(), "immunotherapy".to_string()],
        },
        Article {
            pmid: "87654321".to_string(),
            title: "Novel Approaches to Cancer Treatment".to_string(),
            authors: vec!["Brown, K.".to_string()],
            abstract_text: "Investigating new therapeutic strategies for advanced cancers."
                .to_string(),
            journal: "Cell".to_string(),
            pub_date: "2024-05-20".to_string(),
            mesh_terms: vec!["Neoplasms".to_string(), "Therapeutics".to_string()],
            chemicals: vec![],
            doi: Some("10.1016/j.cell.2024.05.001".to_string()),
            keywords: vec!["cancer".to_string(), "therapy".to_string()],
        },
        Article {
            pmid: "11223344".to_string(),
            title: "Diabetes Management Guidelines".to_string(),
            authors: vec!["Lee, M.".to_string(), "Chen, X.".to_string()],
            abstract_text: "Comprehensive guidelines for type 2 diabetes management.".to_string(),
            journal: "Diabetes Care".to_string(),
            pub_date: "2024-01-10".to_string(),
            mesh_terms: vec!["Diabetes Mellitus".to_string()],
            chemicals: vec!["Insulin".to_string()],
            doi: None,
            keywords: vec!["diabetes".to_string(), "treatment".to_string()],
        },
    ]
}

#[test]
fn test_database_creation() -> Result<()> {
    let db_path = PathBuf::from("test_db_creation.db");

    // Clean up if exists
    let _ = fs::remove_file(&db_path);

    // Create database
    let _db = ArticleDatabase::create(&db_path)?;

    // Verify file exists
    assert!(db_path.exists());

    // Clean up
    fs::remove_file(&db_path)?;

    Ok(())
}

#[test]
fn test_insert_and_retrieve_articles() -> Result<()> {
    let db_path = PathBuf::from("test_db_insert_retrieve.db");

    // Clean up if exists
    let _ = fs::remove_file(&db_path);

    // Create database and insert articles
    let db = ArticleDatabase::create(&db_path)?;
    let articles = create_test_articles();
    db.insert_articles(&articles)?;

    // Retrieve by PMID
    let article1 = db.get_by_pmid("12345678")?;
    assert!(article1.is_some());

    let article1 = article1.unwrap();
    assert_eq!(article1.pmid, "12345678");
    assert_eq!(article1.title, "Cancer Immunotherapy Mechanisms");
    assert!(article1.authors.contains("Smith, J."));
    assert_eq!(article1.doi, Some("10.1038/nm.1234".to_string()));

    // Test non-existent PMID
    let no_article = db.get_by_pmid("99999999")?;
    assert!(no_article.is_none());

    // Clean up
    fs::remove_file(&db_path)?;

    Ok(())
}

#[test]
fn test_search_by_author() -> Result<()> {
    let db_path = PathBuf::from("test_db_author_search.db");

    // Clean up if exists
    let _ = fs::remove_file(&db_path);

    // Create database and insert articles
    let db = ArticleDatabase::create(&db_path)?;
    let articles = create_test_articles();
    db.insert_articles(&articles)?;

    // Search by author
    let results = db.search_by_author("Smith")?;
    assert!(!results.is_empty());
    assert_eq!(results[0].pmid, "12345678");

    // Search for non-existent author
    let no_results = db.search_by_author("Nonexistent")?;
    assert!(no_results.is_empty());

    // Clean up
    fs::remove_file(&db_path)?;

    Ok(())
}

#[test]
fn test_full_text_search() -> Result<()> {
    let db_path = PathBuf::from("test_db_fts.db");

    // Clean up if exists
    let _ = fs::remove_file(&db_path);

    // Create database and insert articles
    let db = ArticleDatabase::create(&db_path)?;
    let articles = create_test_articles();
    db.insert_articles(&articles)?;

    // Full-text search for "checkpoint inhibitors"
    let results = db.full_text_search("checkpoint inhibitors")?;
    assert!(!results.is_empty());
    assert_eq!(results[0].pmid, "12345678");

    // Search for diabetes
    let diabetes_results = db.full_text_search("diabetes")?;
    assert!(!diabetes_results.is_empty());
    assert_eq!(diabetes_results[0].pmid, "11223344");

    // Clean up
    fs::remove_file(&db_path)?;

    Ok(())
}

#[test]
fn test_database_statistics() -> Result<()> {
    let db_path = PathBuf::from("test_db_stats.db");

    // Clean up if exists
    let _ = fs::remove_file(&db_path);

    // Create database and insert articles
    let db = ArticleDatabase::create(&db_path)?;
    let articles = create_test_articles();
    db.insert_articles(&articles)?;

    // Get statistics
    let stats = db.get_stats()?;

    assert_eq!(stats.total_articles, 3);
    assert_eq!(stats.with_doi, 2);
    assert_eq!(stats.with_abstract, 3);

    // Clean up
    fs::remove_file(&db_path)?;

    Ok(())
}

#[test]
fn test_duplicate_pmid_handling() -> Result<()> {
    let db_path = PathBuf::from("test_db_duplicates.db");

    // Clean up if exists
    let _ = fs::remove_file(&db_path);

    // Create database
    let db = ArticleDatabase::create(&db_path)?;

    // Insert article
    let article1 = vec![Article {
        pmid: "12345678".to_string(),
        title: "Original Title".to_string(),
        authors: vec!["Smith, J.".to_string()],
        abstract_text: "Original abstract.".to_string(),
        journal: "Journal A".to_string(),
        pub_date: "2024-01-01".to_string(),
        mesh_terms: vec![],
        chemicals: vec![],
        doi: None,
        keywords: vec![],
    }];

    db.insert_articles(&article1)?;

    // Insert article with same PMID but different data (should replace)
    let article2 = vec![Article {
        pmid: "12345678".to_string(),
        title: "Updated Title".to_string(),
        authors: vec!["Smith, J.".to_string(), "Johnson, A.".to_string()],
        abstract_text: "Updated abstract.".to_string(),
        journal: "Journal B".to_string(),
        pub_date: "2024-02-01".to_string(),
        mesh_terms: vec![],
        chemicals: vec![],
        doi: Some("10.1234/test".to_string()),
        keywords: vec![],
    }];

    db.insert_articles(&article2)?;

    // Verify the article was updated
    let retrieved = db.get_by_pmid("12345678")?.unwrap();
    assert_eq!(retrieved.title, "Updated Title");
    assert_eq!(retrieved.journal, "Journal B");
    assert_eq!(retrieved.doi, Some("10.1234/test".to_string()));

    // Verify only one article exists
    let stats = db.get_stats()?;
    assert_eq!(stats.total_articles, 1);

    // Clean up
    fs::remove_file(&db_path)?;

    Ok(())
}

#[test]
fn test_batch_insert_performance() -> Result<()> {
    let db_path = PathBuf::from("test_db_batch.db");

    // Clean up if exists
    let _ = fs::remove_file(&db_path);

    // Create database
    let db = ArticleDatabase::create(&db_path)?;

    // Create many articles
    let mut articles = Vec::new();
    for i in 0..1000 {
        articles.push(Article {
            pmid: format!("PMID{:08}", i),
            title: format!("Article Title {}", i),
            authors: vec![format!("Author {}", i)],
            abstract_text: format!("Abstract text for article {}", i),
            journal: "Test Journal".to_string(),
            pub_date: "2024-05-27".to_string(),
            mesh_terms: vec!["Test Term".to_string()],
            chemicals: vec![],
            doi: None,
            keywords: vec!["test".to_string()],
        });
    }

    // Insert all articles
    db.insert_articles(&articles)?;

    // Verify count
    let stats = db.get_stats()?;
    assert_eq!(stats.total_articles, 1000);

    // Verify we can retrieve one
    let article = db.get_by_pmid("PMID00000500")?;
    assert!(article.is_some());
    assert_eq!(article.unwrap().title, "Article Title 500");

    // Clean up
    fs::remove_file(&db_path)?;

    Ok(())
}

use curalit::article::Article;
use curalit::statistics::Statistics;
use curalit::visualizer::VisualizationGenerator;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that visualization script is generated successfully
#[test]
fn test_visualization_generation() {
    let temp_dir = TempDir::new().unwrap();
    let stats = create_test_statistics();
    
    let generator = VisualizationGenerator::new(
        stats,
        "test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    let result = generator.generate();
    assert!(result.is_ok(), "Should generate visualization script successfully");
    
    // Check that the file was created
    let script_path = temp_dir.path().join("test_20260620_120000_visualize.py");
    assert!(script_path.exists(), "Visualization script should exist");
    
    // Check that file is not empty
    let content = fs::read_to_string(&script_path).unwrap();
    assert!(!content.is_empty(), "Script should not be empty");
    assert!(content.len() > 1000, "Script should contain substantial content");
}

/// Test that the generated Python script is syntactically valid
#[test]
fn test_python_syntax_validity() {
    let temp_dir = TempDir::new().unwrap();
    let stats = create_test_statistics_with_articles();
    
    let generator = VisualizationGenerator::new(
        stats,
        "syntax_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("syntax_test_20260620_120000_visualize.py");
    
    // Use Python to check syntax
    let output = Command::new("python3")
        .arg("-m")
        .arg("py_compile")
        .arg(&script_path)
        .output();
    
    match output {
        Ok(result) => {
            assert!(
                result.status.success(),
                "Python script should be syntactically valid. stderr: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        Err(e) => {
            eprintln!("Warning: Could not run Python syntax check: {}", e);
            eprintln!("Skipping Python validation (Python 3 may not be available)");
        }
    }
}

/// Test that the script contains all required functions
#[test]
fn test_script_contains_required_functions() {
    let temp_dir = TempDir::new().unwrap();
    let stats = create_test_statistics();
    
    let generator = VisualizationGenerator::new(
        stats,
        "function_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("function_test_20260620_120000_visualize.py");
    let content = fs::read_to_string(&script_path).unwrap();
    
    // Check for required functions
    assert!(content.contains("def create_year_distribution_plot():"), "Should contain year distribution function");
    assert!(content.contains("def create_mesh_terms_plot():"), "Should contain mesh terms function");
    assert!(content.contains("def create_keyword_article_network("), "Should contain network function");
    assert!(content.contains("def main():"), "Should contain main function");
    
    // Check for required imports
    assert!(content.contains("import plotly"), "Should import plotly");
    assert!(content.contains("import pandas"), "Should import pandas");
    assert!(content.contains("from datetime import datetime"), "Should import datetime");
    
    // Check for network-specific imports
    assert!(content.contains("import networkx as nx"), "Should import networkx");
    assert!(content.contains("from pyvis.network import Network"), "Should import PyVis");
}

/// Test that statistics data is properly embedded in the script
#[test]
fn test_statistics_data_embedded() {
    let temp_dir = TempDir::new().unwrap();
    let stats = create_test_statistics();
    
    let generator = VisualizationGenerator::new(
        stats,
        "data_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("data_test_20260620_120000_visualize.py");
    let content = fs::read_to_string(&script_path).unwrap();
    
    // Check for embedded statistics
    assert!(content.contains("TOTAL_ARTICLES = 100"), "Should contain total articles");
    assert!(content.contains("ARTICLES_WITH_ABSTRACTS = 95"), "Should contain abstracts count");
    assert!(content.contains("ARTICLES_WITH_DOI = 90"), "Should contain DOI count");
    
    // Check for data structures
    assert!(content.contains("year_data = {"), "Should contain year data");
    assert!(content.contains("mesh_data = {"), "Should contain mesh data");
    assert!(content.contains("SEARCH_KEYWORDS = ["), "Should contain search keywords");
    assert!(content.contains("ARTICLES_DATA = ["), "Should contain articles data");
}

/// Test that search keywords are properly formatted
#[test]
fn test_search_keywords_formatting() {
    let temp_dir = TempDir::new().unwrap();
    let mut stats = create_test_statistics();
    stats.search_keywords = vec![
        "diabetes".to_string(),
        "insulin".to_string(),
        "glucose".to_string(),
    ];
    
    let generator = VisualizationGenerator::new(
        stats,
        "keywords_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("keywords_test_20260620_120000_visualize.py");
    let content = fs::read_to_string(&script_path).unwrap();
    
    // Check that keywords are in Python list format
    assert!(content.contains("SEARCH_KEYWORDS = ['diabetes', 'insulin', 'glucose']"), 
            "Keywords should be formatted as Python list");
}

/// Test that articles data is properly formatted
#[test]
fn test_articles_data_formatting() {
    let temp_dir = TempDir::new().unwrap();
    let stats = create_test_statistics_with_articles();
    
    let generator = VisualizationGenerator::new(
        stats,
        "articles_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("articles_test_20260620_120000_visualize.py");
    let content = fs::read_to_string(&script_path).unwrap();
    
    // Check that articles are formatted correctly
    assert!(content.contains("'pmid':"), "Should contain pmid field");
    assert!(content.contains("'title':"), "Should contain title field");
    assert!(content.contains("'authors':"), "Should contain authors field");
    assert!(content.contains("'pub_date':"), "Should contain pub_date field");
    assert!(content.contains("'abstract':"), "Should contain abstract field");
    assert!(content.contains("'mesh_terms':"), "Should contain mesh_terms field");
}

/// Test network function parameters
#[test]
fn test_network_function_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let stats = create_test_statistics_with_articles();
    
    let generator = VisualizationGenerator::new(
        stats,
        "network_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("network_test_20260620_120000_visualize.py");
    let content = fs::read_to_string(&script_path).unwrap();
    
    // Check function signature
    assert!(
        content.contains("def create_keyword_article_network(max_articles=None, recent_years=3, show_all=False, use_mesh=False):"),
        "Network function should have correct parameters"
    );
    
    // Check for filtering logic
    assert!(content.contains("filtered_articles = []"), "Should have article filtering");
    assert!(content.contains("for article in ARTICLES_DATA:"), "Should iterate over articles");
    assert!(content.contains("pub_date = article.get('pub_date', '')"), "Should check pub_date");
    assert!(content.contains("if pub_date:"), "Should validate pub_date");
    assert!(content.contains("year_str = pub_date.split('-')[0]"), "Should extract year");
    assert!(content.contains("if year_str.isdigit()"), "Should validate year format");
    assert!(content.contains("if len(filtered_articles) == 0:"), "Should check for empty results");
    assert!(content.contains("No articles from last"), "Should have fallback message");
    
    // Check for PubMed URL generation
    assert!(content.contains("https://pubmed.ncbi.nlm.nih.gov/"), "Should generate PubMed URLs");
}

/// Test that special characters are properly escaped
#[test]
fn test_special_character_escaping() {
    let temp_dir = TempDir::new().unwrap();
    let mut stats = create_test_statistics();
    
    // Add data with special characters
    stats.top_authors = vec![
        ("O'Brien, J".to_string(), 10),
        ("Smith's Lab".to_string(), 5),
    ];
    
    let generator = VisualizationGenerator::new(
        stats,
        "escape_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("escape_test_20260620_120000_visualize.py");
    let content = fs::read_to_string(&script_path).unwrap();
    
    // Check that apostrophes are escaped
    assert!(content.contains("\\'"), "Should escape apostrophes");
}

/// Test visualization script with empty datasets
#[test]
fn test_empty_datasets() {
    let temp_dir = TempDir::new().unwrap();
    let stats = Statistics {
        total_articles: 0,
        keyword_frequencies: HashMap::new(),
        mesh_term_frequencies: HashMap::new(),
        chemical_frequencies: HashMap::new(),
        author_frequencies: HashMap::new(),
        journal_frequencies: HashMap::new(),
        year_distribution: HashMap::new(),
        top_keywords: Vec::new(),
        top_mesh_terms: Vec::new(),
        top_authors: Vec::new(),
        top_journals: Vec::new(),
        avg_authors_per_article: 0.0,
        avg_mesh_terms_per_article: 0.0,
        articles_with_abstracts: 0,
        articles_with_doi: 0,
        search_keywords: Vec::new(),
        articles: Vec::new(),
    };
    
    let generator = VisualizationGenerator::new(
        stats,
        "empty_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    let result = generator.generate();
    assert!(result.is_ok(), "Should handle empty datasets gracefully");
    
    let script_path = temp_dir.path().join("empty_test_20260620_120000_visualize.py");
    assert!(script_path.exists(), "Script should be created even with empty data");
}

/// Test that the script creates output directory
#[test]
fn test_script_creates_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let stats = create_test_statistics();
    
    let generator = VisualizationGenerator::new(
        stats,
        "output_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("output_test_20260620_120000_visualize.py");
    let content = fs::read_to_string(&script_path).unwrap();
    
    // Check that script creates html subdirectory
    assert!(content.contains("html_dir = os.path.join("), "Should define html_dir");
    assert!(content.contains("os.makedirs(html_dir, exist_ok=True)"), "Should create html directory");
}

/// Test network graph generation in main function
#[test]
fn test_network_graph_in_main() {
    let temp_dir = TempDir::new().unwrap();
    let stats = create_test_statistics_with_articles();
    
    let generator = VisualizationGenerator::new(
        stats,
        "main_test",
        temp_dir.path(),
        "20260620_120000",
    );
    
    generator.generate().unwrap();
    
    let script_path = temp_dir.path().join("main_test_20260620_120000_visualize.py");
    let content = fs::read_to_string(&script_path).unwrap();
    
    // Check that main function calls network generation
    assert!(content.contains("network_result = create_keyword_article_network("), 
            "Main should call network function");
    assert!(content.contains("max_articles=100"), "Should set max_articles parameter");
    assert!(content.contains("recent_years=3"), "Should set recent_years parameter");
    assert!(content.contains("_keyword_network.html"), "Should save network HTML");
    assert!(content.contains("net.save_graph(output_path)"), "Should save network graph");
}

// Helper functions for creating test data

fn create_test_statistics() -> Statistics {
    let mut year_dist = HashMap::new();
    year_dist.insert("2023".to_string(), 50);
    year_dist.insert("2024".to_string(), 50);
    
    Statistics {
        total_articles: 100,
        keyword_frequencies: HashMap::new(),
        mesh_term_frequencies: HashMap::new(),
        chemical_frequencies: HashMap::new(),
        author_frequencies: HashMap::new(),
        journal_frequencies: HashMap::new(),
        year_distribution: year_dist,
        top_keywords: Vec::new(),
        top_mesh_terms: vec![("Cancer".to_string(), 50), ("Diabetes".to_string(), 30)],
        top_authors: vec![("Smith, J".to_string(), 10), ("Jones, A".to_string(), 8)],
        top_journals: vec![("Nature".to_string(), 5), ("Science".to_string(), 4)],
        avg_authors_per_article: 3.5,
        avg_mesh_terms_per_article: 8.2,
        articles_with_abstracts: 95,
        articles_with_doi: 90,
        search_keywords: vec!["diabetes".to_string(), "insulin".to_string()],
        articles: Vec::new(),
    }
}

fn create_test_statistics_with_articles() -> Statistics {
    let mut stats = create_test_statistics();
    
    // Add sample articles
    let article1 = Article {
        pmid: "12345678".to_string(),
        title: "Diabetes and Insulin Resistance".to_string(),
        authors: vec!["Smith, J".to_string(), "Jones, A".to_string()],
        abstract_text: "This study examines diabetes and insulin resistance in patients.".to_string(),
        journal: "Nature Medicine".to_string(),
        pub_date: "2024-01-15".to_string(),
        doi: Some("10.1038/s41591-024-12345".to_string()),
        mesh_terms: vec!["Diabetes Mellitus".to_string(), "Insulin Resistance".to_string()],
        chemicals: Vec::new(),
        keywords: vec!["diabetes".to_string(), "insulin".to_string()],
    };
    
    let article2 = Article {
        pmid: "87654321".to_string(),
        title: "Glucose Metabolism in Diabetes".to_string(),
        authors: vec!["Johnson, B".to_string(), "Williams, C".to_string()],
        abstract_text: "Research on glucose metabolism and insulin signaling.".to_string(),
        journal: "Cell Metabolism".to_string(),
        pub_date: "2023-06-20".to_string(),
        doi: Some("10.1016/j.cmet.2023.54321".to_string()),
        mesh_terms: vec!["Glucose".to_string(), "Metabolism".to_string()],
        chemicals: vec!["Glucose".to_string()],
        keywords: vec!["diabetes".to_string(), "glucose".to_string()],
    };
    
    stats.articles = vec![article1, article2];
    stats
}

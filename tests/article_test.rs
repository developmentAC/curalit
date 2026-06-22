//! Comprehensive unit tests for Article functionality
//! Tests keyword matching, CSV operations, and data transformations

use curalit::article::Article;

/// Helper function to create a sample article for testing
fn create_sample_article() -> Article {
    Article {
        pmid: "12345678".to_string(),
        title: "Cancer Immunotherapy Using Checkpoint Inhibitors".to_string(),
        authors: vec![
            "Smith, John".to_string(),
            "Johnson, Alice".to_string(),
            "Williams, Robert".to_string(),
        ],
        abstract_text: "This study investigates the efficacy of PD-1 checkpoint inhibitors in treating melanoma. We conducted a randomized controlled trial with 500 patients showing significant improvement in survival rates.".to_string(),
        journal: "Nature Medicine".to_string(),
        pub_date: "2024-03-15".to_string(),
        mesh_terms: vec![
            "Neoplasms".to_string(),
            "Immunotherapy".to_string(),
            "Checkpoint Inhibitors".to_string(),
            "Melanoma".to_string(),
        ],
        chemicals: vec!["PD-1".to_string(), "Pembrolizumab".to_string()],
        doi: Some("10.1038/nm.2024.1234".to_string()),
        keywords: vec!["cancer".to_string(), "immunotherapy".to_string(), "clinical trial".to_string()],
    }
}

/// Test basic article creation
#[test]
fn test_article_creation() {
    let article = Article::new("99999999".to_string());
    assert_eq!(article.pmid, "99999999");
    assert!(article.title.is_empty());
    assert!(article.authors.is_empty());
    assert!(article.abstract_text.is_empty());
    assert_eq!(article.doi, None);
}

/// Test searchable text generation
#[test]
fn test_searchable_text() {
    let article = create_sample_article();
    let searchable = article.get_searchable_text();

    // Should be lowercase
    assert!(searchable.chars().all(|c| !c.is_uppercase()));

    // Should contain key terms from all fields
    assert!(searchable.contains("cancer"));
    assert!(searchable.contains("immunotherapy"));
    assert!(searchable.contains("smith"));
    assert!(searchable.contains("melanoma"));
    assert!(searchable.contains("pd-1"));
}

/// Test single keyword matching
#[test]
fn test_single_keyword_match() {
    let article = create_sample_article();

    // Matches in title
    assert!(article.matches_keyword("cancer"));
    assert!(article.matches_keyword("immunotherapy"));

    // Matches in abstract
    assert!(article.matches_keyword("melanoma"));
    assert!(article.matches_keyword("survival"));

    // Matches in authors
    assert!(article.matches_keyword("smith"));
    assert!(article.matches_keyword("johnson"));

    // Matches in chemicals
    assert!(article.matches_keyword("pd-1"));
    assert!(article.matches_keyword("pembrolizumab"));

    // Matches in MeSH terms
    assert!(article.matches_keyword("neoplasms"));

    // Case insensitive
    assert!(article.matches_keyword("CANCER"));
    assert!(article.matches_keyword("Cancer"));
    assert!(article.matches_keyword("cAnCeR"));

    // Doesn't match
    assert!(!article.matches_keyword("diabetes"));
    assert!(!article.matches_keyword("glucose"));
}

/// Test AND logic keyword matching
#[test]
fn test_matches_all_keywords_and_logic() {
    let article = create_sample_article();

    // All keywords present
    assert!(article.matches_all_keywords(&["cancer".to_string(), "immunotherapy".to_string(),]));

    assert!(article.matches_all_keywords(&[
        "cancer".to_string(),
        "melanoma".to_string(),
        "pd-1".to_string(),
    ]));

    // One keyword missing
    assert!(!article.matches_all_keywords(&["cancer".to_string(), "diabetes".to_string(),]));

    assert!(!article.matches_all_keywords(&[
        "immunotherapy".to_string(),
        "glucose".to_string(),
        "insulin".to_string(),
    ]));

    // Case insensitive
    assert!(article.matches_all_keywords(&["CANCER".to_string(), "IMMUNOTHERAPY".to_string(),]));

    // Empty keywords (should return true - vacuous truth)
    assert!(article.matches_all_keywords(&[]));
}

/// Test OR logic keyword matching
#[test]
fn test_matches_any_keyword_or_logic() {
    let article = create_sample_article();

    // At least one keyword present
    assert!(article.matches_any_keyword(&["cancer".to_string(), "diabetes".to_string(),]));

    assert!(article.matches_any_keyword(&[
        "glucose".to_string(),
        "insulin".to_string(),
        "melanoma".to_string(),
    ]));

    // All keywords present
    assert!(article.matches_any_keyword(&["cancer".to_string(), "immunotherapy".to_string(),]));

    // No keywords present
    assert!(!article.matches_any_keyword(&[
        "diabetes".to_string(),
        "glucose".to_string(),
        "insulin".to_string(),
    ]));

    // Case insensitive
    assert!(article.matches_any_keyword(&["DIABETES".to_string(), "CANCER".to_string(),]));

    // Empty keywords (should return false)
    assert!(!article.matches_any_keyword(&[]));
}

/// Test CSV serialization
#[test]
fn test_csv_row_conversion() {
    let article = create_sample_article();
    let csv_row = article.to_csv_row();

    assert_eq!(csv_row.len(), 10); // Should have 10 columns
    assert_eq!(csv_row[0], "12345678"); // PMID
    assert_eq!(
        csv_row[1],
        "Cancer Immunotherapy Using Checkpoint Inhibitors"
    ); // Title
    assert!(csv_row[2].contains("Smith, John")); // Authors
    assert!(csv_row[3].contains("melanoma")); // Abstract
    assert_eq!(csv_row[4], "Nature Medicine"); // Journal
    assert_eq!(csv_row[5], "2024-03-15"); // Pub date
    assert!(csv_row[6].contains("Immunotherapy")); // MeSH
    assert!(csv_row[7].contains("PD-1")); // Chemicals
    assert_eq!(csv_row[8], "10.1038/nm.2024.1234"); // DOI
    assert!(csv_row[9].contains("cancer")); // Keywords
}

/// Test CSV headers
#[test]
fn test_csv_headers() {
    let headers = Article::csv_headers();
    assert_eq!(headers.len(), 10);
    assert_eq!(headers[0], "PMID");
    assert_eq!(headers[1], "Title");
    assert_eq!(headers[9], "Keywords");
}

/// Test CSV deserialization
#[test]
fn test_from_csv_row() {
    use csv::StringRecord;

    let record = StringRecord::from(vec![
        "87654321",
        "Diabetes Treatment Study",
        "Brown, Kate; Lee, Michael",
        "This study examines diabetes management strategies.",
        "Diabetes Care",
        "2024-05-20",
        "Diabetes Mellitus; Insulin",
        "Metformin; Insulin",
        "10.2337/dc24.0001",
        "diabetes; treatment; insulin",
    ]);

    let article = Article::from_csv_row(&record).unwrap();

    assert_eq!(article.pmid, "87654321");
    assert_eq!(article.title, "Diabetes Treatment Study");
    assert_eq!(article.authors.len(), 2);
    assert!(article.authors.contains(&"Brown, Kate".to_string()));
    assert_eq!(article.journal, "Diabetes Care");
    assert_eq!(article.mesh_terms.len(), 2);
    assert_eq!(article.chemicals.len(), 2);
    assert_eq!(article.doi, Some("10.2337/dc24.0001".to_string()));
    assert_eq!(article.keywords.len(), 3);
}

/// Test CSV round-trip conversion
#[test]
fn test_csv_round_trip() {
    let original = create_sample_article();
    let csv_row = original.to_csv_row();

    // Convert to StringRecord
    let record = csv::StringRecord::from(csv_row);

    // Convert back to Article
    let restored = Article::from_csv_row(&record).unwrap();

    // Check key fields match
    assert_eq!(restored.pmid, original.pmid);
    assert_eq!(restored.title, original.title);
    assert_eq!(restored.authors, original.authors);
    assert_eq!(restored.abstract_text, original.abstract_text);
    assert_eq!(restored.journal, original.journal);
    assert_eq!(restored.doi, original.doi);
}

/// Test article with missing DOI
#[test]
fn test_article_without_doi() {
    let mut article = create_sample_article();
    article.doi = None;

    let csv_row = article.to_csv_row();
    assert_eq!(csv_row[8], ""); // DOI field should be empty string

    let record = csv::StringRecord::from(csv_row);
    let restored = Article::from_csv_row(&record).unwrap();
    assert_eq!(restored.doi, None);
}

/// Test training data format
#[test]
fn test_training_format() {
    let article = create_sample_article();
    let training_data = article.to_training_format();

    // Should be a JSON object
    assert!(training_data.is_object());

    // Should have expected fields
    assert!(training_data.get("id").is_some());
    assert!(training_data.get("text").is_some());

    // ID should match PMID
    assert_eq!(training_data["id"], "12345678");

    // Text should contain key information
    let text = training_data["text"].as_str().unwrap();
    assert!(text.contains("Cancer Immunotherapy"));
    assert!(text.contains("Smith, John"));
    assert!(text.contains("melanoma"));
    assert!(text.contains("Nature Medicine"));
}

/// Test edge case: empty keywords list
#[test]
fn test_empty_keywords() {
    let article = create_sample_article();

    // AND logic with empty list should return true (vacuous truth)
    assert!(article.matches_all_keywords(&[]));

    // OR logic with empty list should return false
    assert!(!article.matches_any_keyword(&[]));
}

/// Test edge case: article with minimal data
#[test]
fn test_minimal_article() {
    let article = Article {
        pmid: "00000001".to_string(),
        title: "Minimal Article".to_string(),
        authors: vec![],
        abstract_text: String::new(),
        journal: String::new(),
        pub_date: String::new(),
        mesh_terms: vec![],
        chemicals: vec![],
        doi: None,
        keywords: vec![],
    };

    // Should still be searchable
    assert!(article.matches_keyword("minimal"));
    assert!(article.matches_keyword("article"));
    assert!(!article.matches_keyword("cancer"));
}

/// Test partial keyword matching
#[test]
fn test_partial_keyword_matching() {
    let article = create_sample_article();

    // Full word matches
    assert!(article.matches_keyword("cancer"));
    assert!(article.matches_keyword("immunotherapy"));

    // Partial word matches (substring)
    assert!(article.matches_keyword("immuno"));
    assert!(article.matches_keyword("therapy"));
    assert!(article.matches_keyword("melano"));

    // Should NOT match if not a substring
    assert!(!article.matches_keyword("xyz"));
}

/// Test complex multi-keyword scenarios
#[test]
fn test_complex_keyword_scenarios() {
    let article = create_sample_article();

    // Scenario 1: Research on cancer immunotherapy
    assert!(article.matches_all_keywords(&[
        "cancer".to_string(),
        "immunotherapy".to_string(),
        "checkpoint".to_string(),
    ]));

    // Scenario 2: PD-1 inhibitor research
    assert!(article.matches_all_keywords(&["pd-1".to_string(), "inhibitor".to_string(),]));

    // Scenario 3: Clinical trials (should match)
    assert!(article.matches_all_keywords(&["trial".to_string(), "patients".to_string(),]));

    // Scenario 4: Diabetes research (should NOT match)
    assert!(!article.matches_all_keywords(&["diabetes".to_string(), "glucose".to_string(),]));

    // Scenario 5: Broad oncology search with OR
    assert!(article.matches_any_keyword(&[
        "cancer".to_string(),
        "tumor".to_string(),
        "neoplasm".to_string(),
        "carcinoma".to_string(),
    ]));
}

/// Test special characters in keywords
#[test]
fn test_special_characters() {
    let article = create_sample_article();

    // Hyphenated terms
    assert!(article.matches_keyword("pd-1"));
    assert!(article.matches_keyword("checkpoint"));

    // Numbers in keywords
    assert!(article.matches_keyword("pd-1"));
    assert!(article.matches_keyword("500")); // from "500 patients"
}

/// Test multiple authors matching
#[test]
fn test_multiple_authors() {
    let article = create_sample_article();

    // All authors should be searchable
    assert!(article.matches_keyword("smith"));
    assert!(article.matches_keyword("johnson"));
    assert!(article.matches_keyword("williams"));

    // First names too
    assert!(article.matches_keyword("john"));
    assert!(article.matches_keyword("alice"));
    assert!(article.matches_keyword("robert"));

    // Non-existent author
    assert!(!article.matches_keyword("jones"));
}

/// Test getting matched search keywords
#[test]
fn test_get_matched_keywords() {
    let article = create_sample_article();
    
    // Test with keywords that match
    let search_keywords = vec![
        "cancer".to_string(),
        "immunotherapy".to_string(),
        "diabetes".to_string(), // doesn't match
        "melanoma".to_string(),
    ];
    
    let matched = article.get_matched_keywords(&search_keywords);
    assert_eq!(matched.len(), 3);
    assert!(matched.contains(&"cancer".to_string()));
    assert!(matched.contains(&"immunotherapy".to_string()));
    assert!(matched.contains(&"melanoma".to_string()));
    assert!(!matched.contains(&"diabetes".to_string()));
}

/// Test setting matched keywords
#[test]
fn test_set_matched_keywords() {
    let mut article = create_sample_article();
    
    // Set matched keywords based on search
    let search_keywords = vec![
        "cancer".to_string(),
        "immunotherapy".to_string(),
        "diabetes".to_string(), // doesn't match
    ];
    
    article.set_matched_keywords(&search_keywords);
    
    // Now keywords should contain only matched search terms
    assert_eq!(article.keywords.len(), 2);
    assert!(article.keywords.contains(&"cancer".to_string()));
    assert!(article.keywords.contains(&"immunotherapy".to_string()));
    assert!(!article.keywords.contains(&"diabetes".to_string()));
}

/// Test matched keywords with AND logic
#[test]
fn test_matched_keywords_and_logic() {
    let mut article = create_sample_article();
    
    let search_keywords = vec![
        "cancer".to_string(),
        "immunotherapy".to_string(),
        "melanoma".to_string(),
    ];
    
    // Verify all keywords match (AND logic would succeed)
    assert!(article.matches_all_keywords(&search_keywords));
    
    // Set matched keywords
    article.set_matched_keywords(&search_keywords);
    
    // All three should be in the keywords field
    assert_eq!(article.keywords.len(), 3);
}

/// Test matched keywords with OR logic
#[test]
fn test_matched_keywords_or_logic() {
    let mut article = create_sample_article();
    
    let search_keywords = vec![
        "cancer".to_string(),
        "diabetes".to_string(), // doesn't match
        "glucose".to_string(),  // doesn't match
    ];
    
    // Verify at least one matches (OR logic would succeed)
    assert!(article.matches_any_keyword(&search_keywords));
    
    // Set matched keywords
    article.set_matched_keywords(&search_keywords);
    
    // Only "cancer" should be in keywords
    assert_eq!(article.keywords.len(), 1);
    assert_eq!(article.keywords[0], "cancer");
}

/// Test matched keywords case insensitivity
#[test]
fn test_matched_keywords_case_insensitive() {
    let mut article = create_sample_article();
    
    let search_keywords = vec![
        "CANCER".to_string(),
        "ImMuNoThErApY".to_string(),
        "MeLaNoMa".to_string(),
    ];
    
    article.set_matched_keywords(&search_keywords);
    
    // Should match regardless of case
    assert_eq!(article.keywords.len(), 3);
}

/// Test CSV with matched keywords
#[test]
fn test_csv_with_matched_keywords() {
    let mut article = create_sample_article();
    
    // Set matched keywords
    let search_keywords = vec!["cancer".to_string(), "immunotherapy".to_string()];
    article.set_matched_keywords(&search_keywords);
    
    // Convert to CSV
    let csv_row = article.to_csv_row();
    
    // Keywords should be in the last column
    assert_eq!(csv_row.len(), 10);
    assert!(csv_row[9].contains("cancer"));
    assert!(csv_row[9].contains("immunotherapy"));
}


use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a PubMed article with all relevant metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub pmid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub journal: String,
    pub pub_date: String,
    pub mesh_terms: Vec<String>,
    pub chemicals: Vec<String>,
    pub doi: Option<String>,
    pub keywords: Vec<String>,
}

impl Article {
    /// Create a new article
    pub fn new(pmid: String) -> Self {
        Self {
            pmid,
            title: String::new(),
            authors: Vec::new(),
            abstract_text: String::new(),
            journal: String::new(),
            pub_date: String::new(),
            mesh_terms: Vec::new(),
            chemicals: Vec::new(),
            doi: None,
            keywords: Vec::new(),
        }
    }

    /// Get all searchable text fields combined
    pub fn get_searchable_text(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.title,
            self.abstract_text,
            self.authors.join(" "),
            self.mesh_terms.join(" "),
            self.chemicals.join(" "),
            self.keywords.join(" ")
        )
        .to_lowercase()
    }

    /// Check if article matches a keyword in any field
    pub fn matches_keyword(&self, keyword: &str) -> bool {
        let keyword_lower = keyword.to_lowercase();
        let searchable = self.get_searchable_text();
        searchable.contains(&keyword_lower)
    }

    /// Check if article matches all keywords (AND logic)
    pub fn matches_all_keywords(&self, keywords: &[String]) -> bool {
        let searchable = self.get_searchable_text();
        keywords
            .iter()
            .all(|kw| searchable.contains(&kw.to_lowercase()))
    }

    /// Check if article matches any keyword (OR logic)
    pub fn matches_any_keyword(&self, keywords: &[String]) -> bool {
        let searchable = self.get_searchable_text();
        keywords
            .iter()
            .any(|kw| searchable.contains(&kw.to_lowercase()))
    }

    /// Convert article to CSV row format
    pub fn to_csv_row(&self) -> Vec<String> {
        vec![
            self.pmid.clone(),
            self.title.clone(),
            self.authors.join("; "),
            self.abstract_text.clone(),
            self.journal.clone(),
            self.pub_date.clone(),
            self.mesh_terms.join("; "),
            self.chemicals.join("; "),
            self.doi.clone().unwrap_or_default(),
            self.keywords.join("; "),
        ]
    }

    /// Get CSV headers
    pub fn csv_headers() -> Vec<String> {
        vec![
            "PMID".to_string(),
            "Title".to_string(),
            "Authors".to_string(),
            "Abstract".to_string(),
            "Journal".to_string(),
            "Publication Date".to_string(),
            "MeSH Terms".to_string(),
            "Chemicals".to_string(),
            "DOI".to_string(),
            "Keywords".to_string(),
        ]
    }

    /// Create article from CSV row
    pub fn from_csv_row(row: &csv::StringRecord) -> Result<Self, String> {
        if row.len() < 10 {
            return Err("Invalid CSV row: insufficient columns".to_string());
        }

        Ok(Self {
            pmid: row[0].to_string(),
            title: row[1].to_string(),
            authors: row[2]
                .split("; ")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            abstract_text: row[3].to_string(),
            journal: row[4].to_string(),
            pub_date: row[5].to_string(),
            mesh_terms: row[6]
                .split("; ")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            chemicals: row[7]
                .split("; ")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            doi: if row[8].is_empty() {
                None
            } else {
                Some(row[8].to_string())
            },
            keywords: row[9]
                .split("; ")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
        })
    }

    /// Format article for training data (JSONL format)
    pub fn to_training_format(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.pmid,
            "text": format!(
                "Title: {}\n\nAuthors: {}\n\nAbstract: {}\n\nJournal: {}\n\nPublished: {}\n\nMeSH Terms: {}\n\nKeywords: {}",
                self.title,
                self.authors.join(", "),
                self.abstract_text,
                self.journal,
                self.pub_date,
                self.mesh_terms.join(", "),
                self.keywords.join(", ")
            ),
            "metadata": {
                "pmid": self.pmid,
                "doi": self.doi,
                "journal": self.journal,
                "date": self.pub_date
            }
        })
    }
}

impl fmt::Display for Article {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PMID: {} | {} | {} authors | {} MeSH terms",
            self.pmid,
            self.title,
            self.authors.len(),
            self.mesh_terms.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_creation() {
        let article = Article::new("12345".to_string());
        assert_eq!(article.pmid, "12345");
        assert!(article.title.is_empty());
    }

    #[test]
    fn test_keyword_matching() {
        let mut article = Article::new("1".to_string());
        article.title = "Cancer Treatment".to_string();
        article.abstract_text = "Study of immunotherapy".to_string();

        assert!(article.matches_keyword("cancer"));
        assert!(article.matches_keyword("CANCER")); // case insensitive
        assert!(article.matches_keyword("immunotherapy"));
        assert!(!article.matches_keyword("diabetes"));
    }

    #[test]
    fn test_and_logic() {
        let mut article = Article::new("1".to_string());
        article.title = "Cancer Treatment".to_string();
        article.abstract_text = "Study of immunotherapy".to_string();

        let keywords = vec!["cancer".to_string(), "immunotherapy".to_string()];
        assert!(article.matches_all_keywords(&keywords));

        let keywords = vec!["cancer".to_string(), "diabetes".to_string()];
        assert!(!article.matches_all_keywords(&keywords));
    }

    #[test]
    fn test_or_logic() {
        let mut article = Article::new("1".to_string());
        article.title = "Cancer Treatment".to_string();

        let keywords = vec!["cancer".to_string(), "diabetes".to_string()];
        assert!(article.matches_any_keyword(&keywords));

        let keywords = vec!["diabetes".to_string(), "alzheimer".to_string()];
        assert!(!article.matches_any_keyword(&keywords));
    }
}

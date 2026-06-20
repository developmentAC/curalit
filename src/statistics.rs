use crate::article::Article;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Statistics analyzer for article corpus
#[derive(Debug)]
pub struct StatisticsAnalyzer {
    articles: Vec<Article>,
    keywords: Vec<String>,
}

/// Statistical analysis results
#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub total_articles: usize,
    pub keyword_frequencies: HashMap<String, usize>,
    pub mesh_term_frequencies: HashMap<String, usize>,
    pub chemical_frequencies: HashMap<String, usize>,
    pub author_frequencies: HashMap<String, usize>,
    pub journal_frequencies: HashMap<String, usize>,
    pub year_distribution: HashMap<String, usize>,
    pub top_keywords: Vec<(String, usize)>,
    pub top_mesh_terms: Vec<(String, usize)>,
    pub top_authors: Vec<(String, usize)>,
    pub top_journals: Vec<(String, usize)>,
    pub avg_authors_per_article: f64,
    pub avg_mesh_terms_per_article: f64,
    pub articles_with_abstracts: usize,
    pub articles_with_doi: usize,
    pub search_keywords: Vec<String>,
    #[serde(skip)]
    pub articles: Vec<Article>,
}

impl StatisticsAnalyzer {
    pub fn new(articles: Vec<Article>, keywords: Vec<String>) -> Self {
        Self { articles, keywords }
    }

    /// Perform comprehensive statistical analysis
    pub fn analyze(&self) -> Result<Statistics> {
        let mut keyword_freq: HashMap<String, usize> = HashMap::new();
        let mut mesh_freq: HashMap<String, usize> = HashMap::new();
        let mut chemical_freq: HashMap<String, usize> = HashMap::new();
        let mut author_freq: HashMap<String, usize> = HashMap::new();
        let mut journal_freq: HashMap<String, usize> = HashMap::new();
        let mut year_dist: HashMap<String, usize> = HashMap::new();

        let mut total_authors = 0;
        let mut total_mesh_terms = 0;
        let mut articles_with_abstracts = 0;
        let mut articles_with_doi = 0;

        for article in &self.articles {
            // Keyword frequencies (from article keywords)
            for keyword in &article.keywords {
                *keyword_freq.entry(keyword.clone()).or_insert(0) += 1;
            }

            // MeSH term frequencies
            for mesh_term in &article.mesh_terms {
                *mesh_freq.entry(mesh_term.clone()).or_insert(0) += 1;
                total_mesh_terms += 1;
            }

            // Chemical frequencies
            for chemical in &article.chemicals {
                *chemical_freq.entry(chemical.clone()).or_insert(0) += 1;
            }

            // Author frequencies
            for author in &article.authors {
                *author_freq.entry(author.clone()).or_insert(0) += 1;
                total_authors += 1;
            }

            // Journal frequencies
            if !article.journal.is_empty() {
                *journal_freq.entry(article.journal.clone()).or_insert(0) += 1;
            }

            // Year distribution
            let year = self.extract_year(&article.pub_date);
            if !year.is_empty() {
                *year_dist.entry(year).or_insert(0) += 1;
            }

            // Count articles with abstracts
            if !article.abstract_text.is_empty() {
                articles_with_abstracts += 1;
            }

            // Count articles with DOI
            if article.doi.is_some() {
                articles_with_doi += 1;
            }
        }

        // Get top items
        let top_keywords = Self::get_top_n(&keyword_freq, 20);
        let top_mesh_terms = Self::get_top_n(&mesh_freq, 20);
        let top_authors = Self::get_top_n(&author_freq, 20);
        let top_journals = Self::get_top_n(&journal_freq, 20);

        // Calculate averages
        let total = self.articles.len() as f64;
        let avg_authors = if total > 0.0 {
            total_authors as f64 / total
        } else {
            0.0
        };
        let avg_mesh_terms = if total > 0.0 {
            total_mesh_terms as f64 / total
        } else {
            0.0
        };

        Ok(Statistics {
            total_articles: self.articles.len(),
            keyword_frequencies: keyword_freq,
            mesh_term_frequencies: mesh_freq,
            chemical_frequencies: chemical_freq,
            author_frequencies: author_freq,
            journal_frequencies: journal_freq,
            year_distribution: year_dist,
            top_keywords,
            top_mesh_terms,
            top_authors,
            top_journals,
            avg_authors_per_article: avg_authors,
            avg_mesh_terms_per_article: avg_mesh_terms,
            articles_with_abstracts,
            articles_with_doi,
            search_keywords: self.keywords.clone(),
            articles: self.articles.clone(),
        })
    }

    /// Extract year from publication date
    fn extract_year(&self, pub_date: &str) -> String {
        pub_date.split('-').next().unwrap_or("").trim().to_string()
    }

    /// Get top N items from frequency map
    fn get_top_n(freq_map: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
        let mut items: Vec<_> = freq_map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(n);
        items
    }
}

impl Statistics {
    /// Write statistics to a log file
    pub fn write_log(&self, file_path: &str) -> Result<()> {
        let mut log = String::new();

        log.push_str("═══════════════════════════════════════════════════════════\n");
        log.push_str("                 CuraLit Statistics Report\n");
        log.push_str("═══════════════════════════════════════════════════════════\n\n");

        log.push_str(&format!("Total Articles: {}\n", self.total_articles));
        log.push_str(&format!(
            "Articles with Abstracts: {} ({:.1}%)\n",
            self.articles_with_abstracts,
            (self.articles_with_abstracts as f64 / self.total_articles as f64) * 100.0
        ));
        log.push_str(&format!(
            "Articles with DOI: {} ({:.1}%)\n\n",
            self.articles_with_doi,
            (self.articles_with_doi as f64 / self.total_articles as f64) * 100.0
        ));

        log.push_str(&format!(
            "Average Authors per Article: {:.2}\n",
            self.avg_authors_per_article
        ));
        log.push_str(&format!(
            "Average MeSH Terms per Article: {:.2}\n\n",
            self.avg_mesh_terms_per_article
        ));

        // Threshold warning
        if self.total_articles > 1000 {
            log.push_str("⚠ WARNING: Article count exceeds recommended threshold (1000)\n");
            log.push_str("  Consider refining keywords for more focused results.\n\n");
        } else if self.total_articles < 50 {
            log.push_str("⚠ NOTE: Low article count (< 50)\n");
            log.push_str("  Consider using broader keywords or OR logic.\n\n");
        }

        log.push_str("───────────────────────────────────────────────────────────\n");
        log.push_str("Top 20 MeSH Terms:\n");
        log.push_str("───────────────────────────────────────────────────────────\n");
        for (i, (term, count)) in self.top_mesh_terms.iter().enumerate() {
            log.push_str(&format!("{:2}. {:40} {:6}\n", i + 1, term, count));
        }
        log.push_str("\n");

        log.push_str("───────────────────────────────────────────────────────────\n");
        log.push_str("Top 20 Authors:\n");
        log.push_str("───────────────────────────────────────────────────────────\n");
        for (i, (author, count)) in self.top_authors.iter().enumerate() {
            log.push_str(&format!("{:2}. {:40} {:6}\n", i + 1, author, count));
        }
        log.push_str("\n");

        log.push_str("───────────────────────────────────────────────────────────\n");
        log.push_str("Top 20 Journals:\n");
        log.push_str("───────────────────────────────────────────────────────────\n");
        for (i, (journal, count)) in self.top_journals.iter().enumerate() {
            log.push_str(&format!("{:2}. {:40} {:6}\n", i + 1, journal, count));
        }
        log.push_str("\n");

        log.push_str("───────────────────────────────────────────────────────────\n");
        log.push_str("Year Distribution:\n");
        log.push_str("───────────────────────────────────────────────────────────\n");
        let mut years: Vec<_> = self.year_distribution.iter().collect();
        years.sort_by(|a, b| b.0.cmp(a.0)); // Sort by year descending
        for (year, count) in years {
            log.push_str(&format!("{}: {}\n", year, count));
        }
        log.push_str("\n");

        log.push_str("═══════════════════════════════════════════════════════════\n");
        log.push_str("End of Report\n");
        log.push_str("═══════════════════════════════════════════════════════════\n");

        fs::write(file_path, log)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_analysis() {
        let mut article1 = Article::new("1".to_string());
        article1.title = "Cancer Study".to_string();
        article1.authors = vec!["Smith, J".to_string(), "Doe, J".to_string()];
        article1.mesh_terms = vec!["Cancer".to_string(), "Treatment".to_string()];
        article1.pub_date = "2023-01-15".to_string();

        let mut article2 = Article::new("2".to_string());
        article2.title = "Diabetes Research".to_string();
        article2.authors = vec!["Smith, J".to_string()];
        article2.mesh_terms = vec!["Diabetes".to_string()];
        article2.pub_date = "2023-05-20".to_string();

        let articles = vec![article1, article2];
        let keywords = vec!["cancer".to_string(), "diabetes".to_string()];

        let analyzer = StatisticsAnalyzer::new(articles, keywords);
        let stats = analyzer.analyze().unwrap();

        assert_eq!(stats.total_articles, 2);
        assert_eq!(stats.avg_authors_per_article, 1.5);
        assert!(stats.top_mesh_terms.len() > 0);
    }
}

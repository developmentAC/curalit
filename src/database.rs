//! SQLite database module for factual information retrieval
//!
//! This module provides functionality to create and query a SQLite database
//! containing PubMed article information. The database is used to verify
//! factual information (PMID, Authors, DOI, etc.) that AI models often
//! confuse during literature reviews.

use crate::article::Article;
use anyhow::{Context, Result};
use colored::*;
use rusqlite::{params, Connection};
use std::path::Path;

/// Database manager for article information
pub struct ArticleDatabase {
    conn: Connection,
}

impl ArticleDatabase {
    /// Create a new database at the specified path
    pub fn create<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path.as_ref())
            .with_context(|| format!("Failed to create database at {:?}", db_path.as_ref()))?;

        // Create articles table with all relevant fields
        conn.execute(
            "CREATE TABLE IF NOT EXISTS articles (
                pmid TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                authors TEXT,
                abstract TEXT,
                journal TEXT,
                pub_date TEXT,
                doi TEXT,
                mesh_terms TEXT,
                keywords TEXT
            )",
            [],
        )?;

        // Create indexes for faster queries
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pmid ON articles(pmid)", [])?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_authors ON articles(authors)",
            [],
        )?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_doi ON articles(doi)", [])?;

        // Enable full-text search on title and abstract
        // Using standalone FTS5 table (not linked to articles table with content=)
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
                pmid UNINDEXED,
                title,
                abstract
            )",
            [],
        )?;

        println!("{}", "✓ Database schema created".green());

        Ok(Self { conn })
    }

    /// Open an existing database
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path.as_ref())
            .with_context(|| format!("Failed to open database at {:?}", db_path.as_ref()))?;

        Ok(Self { conn })
    }

    /// Insert multiple articles into the database
    pub fn insert_articles(&self, articles: &[Article]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        for article in articles {
            tx.execute(
                "INSERT OR REPLACE INTO articles 
                (pmid, title, authors, abstract, journal, pub_date, doi, mesh_terms, keywords)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    &article.pmid,
                    &article.title,
                    &article.authors.join("; "),
                    &article.abstract_text,
                    &article.journal,
                    &article.pub_date,
                    &article.doi,
                    &article.mesh_terms.join("; "),
                    &article.keywords.join("; "),
                ],
            )?;

            // Update FTS index (FTS5 doesn't support UPSERT, so delete then insert)
            tx.execute(
                "DELETE FROM articles_fts WHERE pmid = ?1",
                params![&article.pmid],
            )?;

            tx.execute(
                "INSERT INTO articles_fts(pmid, title, abstract)
                VALUES (?1, ?2, ?3)",
                params![&article.pmid, &article.title, &article.abstract_text,],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Query article by PMID
    pub fn get_by_pmid(&self, pmid: &str) -> Result<Option<ArticleInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT pmid, title, authors, abstract, journal, pub_date, doi, mesh_terms, keywords
             FROM articles WHERE pmid = ?1",
        )?;

        let result = stmt.query_row(params![pmid], |row| {
            Ok(ArticleInfo {
                pmid: row.get(0)?,
                title: row.get(1)?,
                authors: row.get(2)?,
                abstract_text: row.get(3)?,
                journal: row.get(4)?,
                pub_date: row.get(5)?,
                doi: row.get(6)?,
                mesh_terms: row.get(7)?,
                keywords: row.get(8)?,
            })
        });

        match result {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Search articles by author name
    pub fn search_by_author(&self, author_name: &str) -> Result<Vec<ArticleInfo>> {
        let pattern = format!("%{}%", author_name);
        let mut stmt = self.conn.prepare(
            "SELECT pmid, title, authors, abstract, journal, pub_date, doi, mesh_terms, keywords
             FROM articles WHERE authors LIKE ?1",
        )?;

        let results = stmt.query_map(params![pattern], |row| {
            Ok(ArticleInfo {
                pmid: row.get(0)?,
                title: row.get(1)?,
                authors: row.get(2)?,
                abstract_text: row.get(3)?,
                journal: row.get(4)?,
                pub_date: row.get(5)?,
                doi: row.get(6)?,
                mesh_terms: row.get(7)?,
                keywords: row.get(8)?,
            })
        })?;

        let mut articles = Vec::new();
        for result in results {
            articles.push(result?);
        }

        Ok(articles)
    }

    /// Full-text search across title and abstract
    pub fn full_text_search(&self, query: &str) -> Result<Vec<ArticleInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.pmid, a.title, a.authors, a.abstract, a.journal, a.pub_date, a.doi, a.mesh_terms, a.keywords
             FROM articles a
             JOIN articles_fts fts ON a.pmid = fts.pmid
             WHERE articles_fts MATCH ?1
             ORDER BY rank
             LIMIT 50"
        )?;

        let results = stmt.query_map(params![query], |row| {
            Ok(ArticleInfo {
                pmid: row.get(0)?,
                title: row.get(1)?,
                authors: row.get(2)?,
                abstract_text: row.get(3)?,
                journal: row.get(4)?,
                pub_date: row.get(5)?,
                doi: row.get(6)?,
                mesh_terms: row.get(7)?,
                keywords: row.get(8)?,
            })
        })?;

        let mut articles = Vec::new();
        for result in results {
            articles.push(result?);
        }

        Ok(articles)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let total_articles: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM articles", [], |row| row.get(0))?;

        let with_doi: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM articles WHERE doi IS NOT NULL AND doi != ''",
            [],
            |row| row.get(0),
        )?;

        let with_abstract: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM articles WHERE abstract IS NOT NULL AND abstract != ''",
            [],
            |row| row.get(0),
        )?;

        Ok(DatabaseStats {
            total_articles: total_articles as usize,
            with_doi: with_doi as usize,
            with_abstract: with_abstract as usize,
        })
    }
}

/// Simplified article information from database
#[derive(Debug, Clone)]
pub struct ArticleInfo {
    pub pmid: String,
    pub title: String,
    pub authors: String,
    pub abstract_text: String,
    pub journal: String,
    pub pub_date: String,
    pub doi: Option<String>,
    pub mesh_terms: String,
    pub keywords: String,
}

impl ArticleInfo {
    /// Format article info for display
    pub fn format_citation(&self) -> String {
        format!(
            "PMID: {}\nTitle: {}\nAuthors: {}\nJournal: {}\nDate: {}\nDOI: {}",
            self.pmid,
            self.title,
            self.authors,
            self.journal,
            self.pub_date,
            self.doi.as_ref().unwrap_or(&"N/A".to_string())
        )
    }
}

/// Database statistics
#[derive(Debug)]
pub struct DatabaseStats {
    pub total_articles: usize,
    pub with_doi: usize,
    pub with_abstract: usize,
}

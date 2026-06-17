use crate::article::Article;
use anyhow::{Context, Result};
use csv::{Reader, Writer};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Manages checkpoint files for resumable operations
pub struct CheckpointManager {
    file_path: PathBuf,
    writer: Arc<Mutex<Writer<File>>>,
    article_count: Arc<Mutex<usize>>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new<P: AsRef<Path>>(file_path: P, resume: bool) -> Result<Self> {
        let path = file_path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let (file, is_new) = if resume && path.exists() {
            // Open for append
            let file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&path)
                .with_context(|| format!("Failed to open checkpoint file: {:?}", path))?;
            (file, false)
        } else {
            // Create new file
            let file = File::create(&path)
                .with_context(|| format!("Failed to create checkpoint file: {:?}", path))?;
            (file, true)
        };

        let mut writer = Writer::from_writer(file);

        // Write header if new file
        if is_new {
            writer.write_record(&Article::csv_headers())?;
            writer.flush()?;
        }

        // Count existing articles if resuming
        let article_count = if resume && !is_new {
            Self::count_existing_articles(&path)?
        } else {
            0
        };

        Ok(Self {
            file_path: path,
            writer: Arc::new(Mutex::new(writer)),
            article_count: Arc::new(Mutex::new(article_count)),
        })
    }

    /// Load an existing checkpoint file
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let path = file_path.as_ref().to_path_buf();

        if !path.exists() {
            anyhow::bail!("Checkpoint file not found: {:?}", path);
        }

        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open checkpoint file: {:?}", path))?;

        let writer = Writer::from_writer(file);
        let article_count = Self::count_existing_articles(&path)?;

        Ok(Self {
            file_path: path,
            writer: Arc::new(Mutex::new(writer)),
            article_count: Arc::new(Mutex::new(article_count)),
        })
    }

    /// Add an article to the checkpoint
    pub fn add_article(&self, article: &Article) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_record(&article.to_csv_row())?;
        writer.flush()?;

        let mut count = self.article_count.lock().unwrap();
        *count += 1;

        Ok(())
    }

    /// Finalize the checkpoint (flush and close)
    pub fn finalize(&self) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()?;
        Ok(())
    }

    /// Get the number of articles in checkpoint
    pub fn article_count(&self) -> usize {
        *self.article_count.lock().unwrap()
    }

    /// Load all articles from checkpoint
    pub fn load_articles(&self) -> Result<Vec<Article>> {
        let file = File::open(&self.file_path)
            .with_context(|| format!("Failed to open checkpoint: {:?}", self.file_path))?;
        let mut reader = Reader::from_reader(file);

        let mut articles = Vec::new();
        for result in reader.records() {
            let record = result?;
            match Article::from_csv_row(&record) {
                Ok(article) => articles.push(article),
                Err(e) => log::warn!("Failed to parse article from CSV: {}", e),
            }
        }

        Ok(articles)
    }

    /// Count existing articles in checkpoint file
    fn count_existing_articles(path: &Path) -> Result<usize> {
        let file = File::open(path)?;
        let mut reader = Reader::from_reader(file);
        let count = reader.records().count();
        Ok(count)
    }

    /// Get the checkpoint file path
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_checkpoint_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = CheckpointManager::new(temp_file.path(), false).unwrap();
        assert_eq!(manager.article_count(), 0);
    }

    #[test]
    fn test_add_article() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = CheckpointManager::new(temp_file.path(), false).unwrap();

        let mut article = Article::new("12345".to_string());
        article.title = "Test Article".to_string();

        manager.add_article(&article).unwrap();
        assert_eq!(manager.article_count(), 1);
    }

    #[test]
    fn test_load_articles() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = CheckpointManager::new(temp_file.path(), false).unwrap();

        let mut article = Article::new("12345".to_string());
        article.title = "Test Article".to_string();
        article.authors = vec!["Smith, J".to_string()];

        manager.add_article(&article).unwrap();
        manager.finalize().unwrap();

        let loaded = manager.load_articles().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].pmid, "12345");
        assert_eq!(loaded[0].title, "Test Article");
    }

    #[test]
    fn test_resume() {
        let temp_file = NamedTempFile::new().unwrap();

        // Create initial checkpoint
        {
            let manager = CheckpointManager::new(temp_file.path(), false).unwrap();
            let mut article = Article::new("1".to_string());
            article.title = "First".to_string();
            manager.add_article(&article).unwrap();
            manager.finalize().unwrap();
        }

        // Resume and add more
        {
            let manager = CheckpointManager::new(temp_file.path(), true).unwrap();
            assert_eq!(manager.article_count(), 1);

            let mut article = Article::new("2".to_string());
            article.title = "Second".to_string();
            manager.add_article(&article).unwrap();
            manager.finalize().unwrap();
        }

        // Verify
        let manager = CheckpointManager::load(temp_file.path()).unwrap();
        let articles = manager.load_articles().unwrap();
        assert_eq!(articles.len(), 2);
    }
}

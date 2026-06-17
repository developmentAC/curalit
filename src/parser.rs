use crate::article::Article;
use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Streaming XML parser for PubMed articles
/// Uses quick-xml for efficient parsing of large XML files
pub struct PubMedParser {
    reader: Reader<BufReader<File>>,
    current_article: Option<Article>,
    current_tag_stack: Vec<String>,
    current_text: String,
}

impl PubMedParser {
    /// Create a new parser for the given XML file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open file: {:?}", path.as_ref()))?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::from_reader(buf_reader);
        reader.config_mut().trim_text(true);

        Ok(Self {
            reader,
            current_article: None,
            current_tag_stack: Vec::new(),
            current_text: String::new(),
        })
    }

    /// Parse next article from the XML stream
    /// Returns None when no more articles are available
    pub fn next_article(&mut self) -> Result<Option<Article>> {
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    self.current_tag_stack.push(tag_name.clone());

                    // Start of new article
                    if tag_name == "PubmedArticle" {
                        self.current_article = None;
                        self.current_text.clear();
                    }
                }

                Ok(Event::Text(e)) => {
                    self.current_text = e.unescape().unwrap_or_default().trim().to_string();
                }

                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if !self.current_tag_stack.is_empty() {
                        self.current_tag_stack.pop();
                    }

                    self.process_end_tag(&tag_name);

                    // End of article - return it
                    if tag_name == "PubmedArticle" {
                        if let Some(article) = self.current_article.take() {
                            buf.clear();
                            return Ok(Some(article));
                        }
                    }

                    self.current_text.clear();
                }

                Ok(Event::Eof) => {
                    return Ok(None);
                }

                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "XML parsing error at position {}: {:?}",
                        self.reader.buffer_position(),
                        e
                    ));
                }

                _ => {}
            }
            buf.clear();
        }
    }

    /// Process end tag and populate article fields
    fn process_end_tag(&mut self, tag_name: &str) {
        if self.current_text.is_empty() {
            return;
        }

        // Ensure article exists
        if self.is_inside_article() && self.current_article.is_none() {
            // Try to get PMID from context if available
            if tag_name == "PMID" {
                self.current_article = Some(Article::new(self.current_text.clone()));
                return;
            }
        }

        // Check contexts before mutably borrowing article
        let is_inside_author = self.is_inside("Author");
        let is_inside_journal = self.is_inside("Journal");
        let is_inside_pubdate = self.is_inside("PubDate");
        let is_inside_meshheading = self.is_inside("MeshHeading");
        let is_inside_articleidlist = self.current_tag_stack.iter().any(|t| t == "ArticleIdList");

        let article = match self.current_article.as_mut() {
            Some(a) => a,
            None => return,
        };

        match tag_name {
            "PMID" => {
                if article.pmid.is_empty() {
                    article.pmid = self.current_text.clone();
                }
            }
            "ArticleTitle" => {
                article.title = self.current_text.clone();
            }
            "AbstractText" => {
                if !article.abstract_text.is_empty() {
                    article.abstract_text.push(' ');
                }
                article.abstract_text.push_str(&self.current_text);
            }
            "LastName" | "ForeName" | "Initials" => {
                if is_inside_author {
                    // Collect author information
                    let author_name = self.current_text.clone();
                    if !author_name.is_empty() {
                        // Check if this is part of an existing author or new author
                        if tag_name == "LastName" {
                            article.authors.push(author_name);
                        } else if tag_name == "ForeName" && !article.authors.is_empty() {
                            let last_idx = article.authors.len() - 1;
                            article.authors[last_idx] =
                                format!("{}, {}", article.authors[last_idx], author_name);
                        }
                    }
                }
            }
            "Title" => {
                if is_inside_journal {
                    article.journal = self.current_text.clone();
                }
            }
            "Year" | "Month" | "Day" => {
                if is_inside_pubdate {
                    if !article.pub_date.is_empty() {
                        article.pub_date.push('-');
                    }
                    article.pub_date.push_str(&self.current_text);
                }
            }
            "DescriptorName" => {
                if is_inside_meshheading {
                    article.mesh_terms.push(self.current_text.clone());
                }
            }
            "NameOfSubstance" => {
                article.chemicals.push(self.current_text.clone());
            }
            "Keyword" => {
                article.keywords.push(self.current_text.clone());
            }
            "ArticleId" => {
                if is_inside_articleidlist {
                    // This might be a DOI
                    if self.current_text.contains('/') || self.current_text.starts_with("10.") {
                        article.doi = Some(self.current_text.clone());
                    }
                }
            }
            _ => {}
        }
    }

    /// Check if we're inside an article tag
    fn is_inside_article(&self) -> bool {
        self.current_tag_stack
            .iter()
            .any(|tag| tag == "PubmedArticle")
    }

    /// Check if we're inside a specific tag
    fn is_inside(&self, tag_name: &str) -> bool {
        self.current_tag_stack.iter().any(|tag| tag == tag_name)
    }
}

/// Count articles in an XML file without fully parsing
pub fn count_articles<P: AsRef<Path>>(path: P) -> Result<usize> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("Failed to open file: {:?}", path.as_ref()))?;
    let buf_reader = BufReader::new(file);
    let mut reader = Reader::from_reader(buf_reader);
    reader.config_mut().trim_text(true);

    let mut count = 0;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "PubmedArticle" {
                    count += 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!("Error counting articles: {:?}", e));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_xml() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"<?xml version="1.0" encoding="utf-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>12345</PMID>
      <Article>
        <ArticleTitle>Test Article</ArticleTitle>
        <Abstract>
          <AbstractText>This is a test abstract.</AbstractText>
        </Abstract>
        <AuthorList>
          <Author>
            <LastName>Smith</LastName>
            <ForeName>John</ForeName>
          </Author>
        </AuthorList>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName>Cancer</DescriptorName>
        </MeshHeading>
      </MeshHeadingList>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>"#
        )
        .unwrap();
        file
    }

    #[test]
    fn test_parse_article() {
        let file = create_test_xml();
        let mut parser = PubMedParser::new(file.path()).unwrap();

        let article = parser.next_article().unwrap();
        assert!(article.is_some());

        let article = article.unwrap();
        assert_eq!(article.pmid, "12345");
        assert_eq!(article.title, "Test Article");
        assert!(article.abstract_text.contains("test abstract"));
        assert_eq!(article.authors.len(), 1);
        assert!(article.mesh_terms.contains(&"Cancer".to_string()));
    }

    #[test]
    fn test_count_articles() {
        let file = create_test_xml();
        let count = count_articles(file.path()).unwrap();
        assert_eq!(count, 1);
    }
}

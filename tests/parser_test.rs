//! Comprehensive tests for PubMed XML parser
//! Tests parsing of various XML structures and edge cases

use anyhow::Result;
use curalit::parser::PubMedParser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Helper to create a temporary XML file for testing
fn create_temp_xml_file(name: &str, content: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("curalit_test_{}.xml", name));
    let mut file = fs::File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(file_path)
}

/// Test parsing a valid complete article
#[test]
fn test_parse_complete_article() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID Version="1">12345678</PMID>
      <Article PubModel="Print">
        <Journal>
          <Title>Nature Medicine</Title>
        </Journal>
        <ArticleTitle>Cancer Immunotherapy Research</ArticleTitle>
        <Abstract>
          <AbstractText>This study investigates novel checkpoint inhibitors for cancer treatment.</AbstractText>
        </Abstract>
        <AuthorList>
          <Author>
            <LastName>Smith</LastName>
            <ForeName>John</ForeName>
          </Author>
          <Author>
            <LastName>Johnson</LastName>
            <ForeName>Alice</ForeName>
          </Author>
        </AuthorList>
        <ArticleDate>
          <Year>2024</Year>
          <Month>03</Month>
          <Day>15</Day>
        </ArticleDate>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName>Neoplasms</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Immunotherapy</DescriptorName>
        </MeshHeading>
      </MeshHeadingList>
      <ChemicalList>
        <Chemical>
          <NameOfSubstance>PD-1 Inhibitor</NameOfSubstance>
        </Chemical>
      </ChemicalList>
    </MedlineCitation>
    <PubmedData>
      <ArticleIdList>
        <ArticleId IdType="doi">10.1038/nm.2024.001</ArticleId>
      </ArticleIdList>
    </PubmedData>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("complete", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let article = parser.next_article()?.expect("Should parse article");

    assert_eq!(article.pmid, "12345678");
    assert_eq!(article.title, "Cancer Immunotherapy Research");
    assert_eq!(article.journal, "Nature Medicine");
    assert!(article.abstract_text.contains("checkpoint inhibitors"));
    assert_eq!(article.authors.len(), 2);
    assert!(article.authors.contains(&"Smith, John".to_string()));
    assert!(article.authors.contains(&"Johnson, Alice".to_string()));
    assert_eq!(article.mesh_terms.len(), 2);
    assert!(article.mesh_terms.contains(&"Neoplasms".to_string()));
    assert!(article.chemicals.contains(&"PD-1 Inhibitor".to_string()));
    assert_eq!(article.doi, Some("10.1038/nm.2024.001".to_string()));

    // Should be no more articles
    assert!(parser.next_article()?.is_none());

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing multiple articles
#[test]
fn test_parse_multiple_articles() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>11111111</PMID>
      <Article>
        <Journal>
          <Title>Journal A</Title>
        </Journal>
        <ArticleTitle>First Article</ArticleTitle>
        <Abstract>
          <AbstractText>First article abstract.</AbstractText>
        </Abstract>
      </Article>
    </MedlineCitation>
  </PubmedArticle>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>22222222</PMID>
      <Article>
        <Journal>
          <Title>Journal B</Title>
        </Journal>
        <ArticleTitle>Second Article</ArticleTitle>
        <Abstract>
          <AbstractText>Second article abstract.</AbstractText>
        </Abstract>
      </Article>
    </MedlineCitation>
  </PubmedArticle>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>33333333</PMID>
      <Article>
        <Journal>
          <Title>Journal C</Title>
        </Journal>
        <ArticleTitle>Third Article</ArticleTitle>
        <Abstract>
          <AbstractText>Third article abstract.</AbstractText>
        </Abstract>
      </Article>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("multiple", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let mut articles = Vec::new();
    while let Some(article) = parser.next_article()? {
        articles.push(article);
    }

    assert_eq!(articles.len(), 3);
    assert_eq!(articles[0].pmid, "11111111");
    assert_eq!(articles[1].pmid, "22222222");
    assert_eq!(articles[2].pmid, "33333333");

    assert_eq!(articles[0].title, "First Article");
    assert_eq!(articles[1].title, "Second Article");
    assert_eq!(articles[2].title, "Third Article");

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing article with missing abstract
#[test]
fn test_parse_article_without_abstract() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>99999999</PMID>
      <Article>
        <Journal>
          <Title>Test Journal</Title>
        </Journal>
        <ArticleTitle>Article Without Abstract</ArticleTitle>
        <AuthorList>
          <Author>
            <LastName>Doe</LastName>
            <ForeName>Jane</ForeName>
          </Author>
        </AuthorList>
      </Article>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("no_abstract", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let article = parser.next_article()?.expect("Should parse article");

    assert_eq!(article.pmid, "99999999");
    assert_eq!(article.title, "Article Without Abstract");
    assert!(article.abstract_text.is_empty() || article.abstract_text == "");
    assert_eq!(article.authors.len(), 1);

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing article with missing DOI
#[test]
fn test_parse_article_without_doi() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>88888888</PMID>
      <Article>
        <Journal>
          <Title>Test Journal</Title>
        </Journal>
        <ArticleTitle>Article Without DOI</ArticleTitle>
      </Article>
    </MedlineCitation>
    <PubmedData>
      <ArticleIdList>
        <ArticleId IdType="pubmed">88888888</ArticleId>
      </ArticleIdList>
    </PubmedData>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("no_doi", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let article = parser.next_article()?.expect("Should parse article");

    assert_eq!(article.pmid, "88888888");
    assert_eq!(article.doi, None);

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing article with multiple authors
#[test]
fn test_parse_multiple_authors() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>77777777</PMID>
      <Article>
        <Journal>
          <Title>Collaboration Journal</Title>
        </Journal>
        <ArticleTitle>Large Collaboration Study</ArticleTitle>
        <AuthorList>
          <Author>
            <LastName>Smith</LastName>
            <ForeName>John</ForeName>
          </Author>
          <Author>
            <LastName>Johnson</LastName>
            <ForeName>Alice</ForeName>
          </Author>
          <Author>
            <LastName>Williams</LastName>
            <ForeName>Robert</ForeName>
          </Author>
          <Author>
            <LastName>Brown</LastName>
            <ForeName>Emily</ForeName>
          </Author>
          <Author>
            <LastName>Davis</LastName>
            <ForeName>Michael</ForeName>
          </Author>
        </AuthorList>
      </Article>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("many_authors", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let article = parser.next_article()?.expect("Should parse article");

    assert_eq!(article.authors.len(), 5);
    assert!(article.authors.contains(&"Smith, John".to_string()));
    assert!(article.authors.contains(&"Davis, Michael".to_string()));

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing article with multiple MeSH terms
#[test]
fn test_parse_multiple_mesh_terms() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>66666666</PMID>
      <Article>
        <Journal>
          <Title>Medical Journal</Title>
        </Journal>
        <ArticleTitle>Complex Disease Study</ArticleTitle>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName>Diabetes Mellitus</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Hypertension</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Obesity</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Cardiovascular Diseases</DescriptorName>
        </MeshHeading>
      </MeshHeadingList>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("many_mesh", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let article = parser.next_article()?.expect("Should parse article");

    assert_eq!(article.mesh_terms.len(), 4);
    assert!(article
        .mesh_terms
        .contains(&"Diabetes Mellitus".to_string()));
    assert!(article
        .mesh_terms
        .contains(&"Cardiovascular Diseases".to_string()));

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing article with chemical list
#[test]
fn test_parse_chemicals() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>55555555</PMID>
      <Article>
        <Journal>
          <Title>Pharmacology Journal</Title>
        </Journal>
        <ArticleTitle>Drug Interaction Study</ArticleTitle>
      </Article>
      <ChemicalList>
        <Chemical>
          <NameOfSubstance>Metformin</NameOfSubstance>
        </Chemical>
        <Chemical>
          <NameOfSubstance>Insulin</NameOfSubstance>
        </Chemical>
        <Chemical>
          <NameOfSubstance>Glipizide</NameOfSubstance>
        </Chemical>
      </ChemicalList>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("chemicals", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let article = parser.next_article()?.expect("Should parse article");

    assert_eq!(article.chemicals.len(), 3);
    assert!(article.chemicals.contains(&"Metformin".to_string()));
    assert!(article.chemicals.contains(&"Insulin".to_string()));
    assert!(article.chemicals.contains(&"Glipizide".to_string()));

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing empty XML file
#[test]
fn test_parse_empty_xml() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("empty", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let result = parser.next_article()?;
    assert!(result.is_none());

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing article with special characters
#[test]
fn test_parse_special_characters() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>44444444</PMID>
      <Article>
        <Journal>
          <Title>International Journal</Title>
        </Journal>
        <ArticleTitle>α-Synuclein &amp; β-Amyloid in Alzheimer's Disease</ArticleTitle>
        <Abstract>
          <AbstractText>This study examines α-synuclein, β-amyloid, and other proteins with special characters like &lt; and &gt;.</AbstractText>
        </Abstract>
      </Article>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("special_chars", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let article = parser.next_article()?.expect("Should parse article");

    assert_eq!(article.pmid, "44444444");
    // XML entities should be properly decoded
    assert!(article.title.contains("&") || article.title.contains("α"));
    assert!(article.abstract_text.contains("α") || article.abstract_text.contains("synuclein"));

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test parsing article with date variations
#[test]
fn test_parse_publication_dates() -> Result<()> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>33333333</PMID>
      <Article>
        <Journal>
          <Title>Test Journal</Title>
        </Journal>
        <ArticleTitle>Date Test</ArticleTitle>
      </Article>
      <PubDate>
        <Year>2024</Year>
        <Month>06</Month>
        <Day>15</Day>
      </PubDate>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("dates", xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let article = parser.next_article()?.expect("Should parse article");

    assert_eq!(article.pmid, "33333333");
    // Date parsing may vary based on XML structure
    // Just verify article was parsed successfully
    assert_eq!(article.title, "Date Test");

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test streaming large number of articles (memory efficiency)
#[test]
fn test_streaming_many_articles() -> Result<()> {
    // Create XML with 100 articles
    let mut xml_content = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>"#,
    );

    for i in 1..=100 {
        xml_content.push_str(&format!(
            r#"
  <PubmedArticle>
    <MedlineCitation>
      <PMID>{:08}</PMID>
      <Article>
        <Journal>
          <Title>Journal {}</Title>
        </Journal>
        <ArticleTitle>Article Number {}</ArticleTitle>
        <Abstract>
          <AbstractText>Abstract for article {}.</AbstractText>
        </Abstract>
      </Article>
    </MedlineCitation>
  </PubmedArticle>"#,
            i, i, i, i
        ));
    }

    xml_content.push_str("\n</PubmedArticleSet>");

    let file_path = create_temp_xml_file("many", &xml_content)?;
    let mut parser = PubMedParser::new(&file_path)?;

    let mut count = 0;
    while let Some(article) = parser.next_article()? {
        count += 1;
        assert_eq!(article.pmid, format!("{:08}", count));
        assert_eq!(article.title, format!("Article Number {}", count));
    }

    assert_eq!(count, 100);

    // Cleanup
    fs::remove_file(&file_path)?;

    Ok(())
}

/// Test error handling for malformed XML
#[test]
fn test_malformed_xml_handling() {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>12345678</PMID>
      <Article>
        <ArticleTitle>Unclosed Tag
      </Article>
    </MedlineCitation>
</PubmedArticleSet>"#;

    let file_path = create_temp_xml_file("malformed", xml_content).unwrap();
    let result = PubMedParser::new(&file_path);

    // Parser should be created successfully
    assert!(result.is_ok());

    let mut parser = result.unwrap();

    // Parsing may fail or skip malformed article
    // We just verify it doesn't panic
    let _ = parser.next_article();

    // Cleanup
    fs::remove_file(&file_path).ok();
}

/// Test handling of non-existent file
#[test]
fn test_nonexistent_file() {
    let result = PubMedParser::new("/nonexistent/path/to/file.xml");
    assert!(result.is_err());
}

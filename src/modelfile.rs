use crate::article::Article;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generator for Ollama Modelfiles and training data
pub struct ModelfileGenerator {
    model_name: String,
    base_model: String,
}

impl ModelfileGenerator {
    pub fn new(model_name: String, base_model: String) -> Self {
        Self {
            model_name,
            base_model,
        }
    }

    /// Generate Modelfile and training data
    pub fn generate(
        &self,
        articles: &[Article],
        output_prefix: &str,
        output_dir: &Path,
        timestamp: &str,
    ) -> Result<()> {
        // Generate training data in JSONL format
        let training_file =
            output_dir.join(format!("{}_{}_training.jsonl", output_prefix, timestamp));
        self.generate_training_data(articles, training_file.to_str().unwrap())?;
        println!(
            "  {} Training data: {}",
            "→".cyan_string(),
            training_file.display()
        );

        // Generate Modelfile
        let modelfile_path =
            output_dir.join(format!("Modelfile_{}_{}", self.model_name, timestamp));
        self.generate_modelfile(
            training_file.to_str().unwrap(),
            modelfile_path.to_str().unwrap(),
        )?;
        println!(
            "  {} Modelfile: {}",
            "→".cyan_string(),
            modelfile_path.display()
        );

        // Generate system prompt
        let system_prompt = self.create_system_prompt(articles);
        let prompt_file =
            output_dir.join(format!("{}_{}_system_prompt.txt", output_prefix, timestamp));
        fs::write(&prompt_file, system_prompt)?;
        println!(
            "  {} System prompt: {}",
            "→".cyan_string(),
            prompt_file.display()
        );

        Ok(())
    }

    /// Generate training data in JSONL format
    fn generate_training_data(&self, articles: &[Article], output_file: &str) -> Result<()> {
        let mut jsonl = String::new();

        for article in articles {
            let training_entry = article.to_training_format();
            jsonl.push_str(&serde_json::to_string(&training_entry)?);
            jsonl.push('\n');
        }

        fs::write(output_file, jsonl)?;
        Ok(())
    }

    /// Generate Ollama Modelfile
    fn generate_modelfile(&self, training_file: &str, output_file: &str) -> Result<()> {
        let modelfile = format!(
            r#"# CuraLit Generated Modelfile
# Model: {}
# Base: {}

FROM {}

# System prompt defining the model's role
SYSTEM """
You are a specialized research assistant trained on a curated corpus of biomedical literature from PubMed. 
Your knowledge base consists of peer-reviewed scientific articles covering specific topics selected by the user.

Your capabilities include:
- Answering questions about the articles in your training corpus
- Explaining complex biomedical concepts for both novices and experts
- Helping identify research topics and formulate hypotheses
- Assisting with literature review synthesis
- Providing citations and references from your knowledge base
- Comparing and contrasting findings across different studies

When answering:
1. Always ground your responses in the articles you were trained on
2. Cite PMIDs when referencing specific articles
3. Acknowledge limitations of your knowledge base
4. Explain concepts at an appropriate level for the user
5. Distinguish between established findings and areas of ongoing research

Remember: Your knowledge is limited to the articles in your training corpus. 
If asked about topics outside this scope, clearly state this limitation.
"""

# Model parameters
PARAMETER temperature 0.7
PARAMETER top_p 0.9
PARAMETER top_k 40
PARAMETER num_ctx 4096

# License and attribution
TEMPLATE """
{{{{ .System }}}}

{{{{ .Prompt }}}}
"""

# Note: Training data is available in {}
# You may need to fine-tune this model using additional tools or scripts
"#,
            self.model_name, self.base_model, self.base_model, training_file
        );

        fs::write(output_file, modelfile)?;
        Ok(())
    }

    /// Create a comprehensive system prompt
    fn create_system_prompt(&self, articles: &[Article]) -> String {
        let article_count = articles.len();

        // Collect unique topics (MeSH terms)
        let mut all_mesh_terms: Vec<String> =
            articles.iter().flat_map(|a| a.mesh_terms.clone()).collect();
        all_mesh_terms.sort();
        all_mesh_terms.dedup();

        let top_mesh = all_mesh_terms
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");

        // Collect date range
        let years: Vec<String> = articles
            .iter()
            .filter_map(|a| {
                a.pub_date
                    .split('-')
                    .next()
                    .and_then(|y| y.parse::<i32>().ok())
            })
            .map(|y| y.to_string())
            .collect();

        let min_year = years.iter().min().cloned().unwrap_or_default();
        let max_year = years.iter().max().cloned().unwrap_or_default();

        format!(
            r#"You are a specialized research assistant with expertise in biomedical literature.

TRAINING CORPUS DETAILS:
- Total Articles: {}
- Date Range: {} - {}
- Primary Topics: {}
- Source: PubMed database

YOUR ROLE:
You help researchers, students, and professionals understand and synthesize information 
from the scientific literature in your training corpus. You can explain concepts for 
both novices and experts, assist with hypothesis generation, and support literature reviews.

CAPABILITIES:
1. Answer specific questions about articles and findings
2. Explain biomedical concepts and terminology
3. Compare methodologies and results across studies
4. Identify research gaps and suggest potential research directions
5. Synthesize information from multiple articles
6. Provide PMID citations for referenced information

LIMITATIONS:
- Your knowledge is limited to the {} articles in your training corpus
- You cannot access new research published after your training
- You cannot provide medical advice or diagnoses
- You should always cite sources (PMIDs) when making specific claims

INTERACTION STYLE:
- Adjust explanation depth based on user's apparent expertise
- Always distinguish facts from interpretations
- Acknowledge uncertainty when appropriate
- Encourage critical thinking about research findings
"#,
            article_count, min_year, max_year, top_mesh, article_count
        )
    }
}

// Helper trait for colored strings
trait ColoredString {
    fn cyan_string(&self) -> String;
}

impl ColoredString for &str {
    fn cyan_string(&self) -> String {
        use colored::Colorize;
        self.cyan().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_modelfile_generation() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path();

        let mut article = Article::new("12345".to_string());
        article.title = "Test Article".to_string();
        article.abstract_text = "Test abstract".to_string();

        let generator = ModelfileGenerator::new("test-model".to_string(), "llama3".to_string());

        let result = generator.generate(&[article], "test", output_path, "20260522_120000");
        assert!(result.is_ok());
    }

    #[test]
    fn test_system_prompt_generation() {
        let mut article = Article::new("1".to_string());
        article.mesh_terms = vec!["Cancer".to_string(), "Immunotherapy".to_string()];
        article.pub_date = "2023".to_string();

        let generator = ModelfileGenerator::new("test".to_string(), "llama3".to_string());
        let prompt = generator.create_system_prompt(&[article]);

        assert!(prompt.contains("Total Articles: 1"));
        assert!(prompt.contains("2023"));
    }
}

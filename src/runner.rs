use crate::checkpoint::CheckpointManager;
use crate::cli::KeywordLogic;
use crate::modelfile::ModelfileGenerator;
use crate::parser::{count_articles, PubMedParser};
use crate::statistics::StatisticsAnalyzer;
use crate::visualizer::VisualizationGenerator;
use anyhow::{Context, Result};
use chrono::Local;
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

pub struct CuraLitRunner {
    keywords: Vec<String>,
    data_dir: PathBuf,
    output_name: String,
    logic: KeywordLogic,
    #[allow(dead_code)]
    resume: bool,
    threshold: usize,
    checkpoint_manager: CheckpointManager,
    output_dir: PathBuf,
}

impl CuraLitRunner {
    /// Create a new runner from CLI parameters
    pub fn new(
        keywords: Vec<String>,
        keywords_file: Option<PathBuf>,
        data_dir: PathBuf,
        output_name: String,
        logic: KeywordLogic,
        resume: bool,
        threshold: usize,
    ) -> Result<Self> {
        // Load keywords from file if provided
        let mut all_keywords = keywords;
        if let Some(file_path) = keywords_file {
            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read keywords file: {:?}", file_path))?;
            all_keywords.extend(
                content
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty()),
            );
        }

        if all_keywords.is_empty() {
            anyhow::bail!("No keywords provided. Use -k or -f to specify keywords.");
        }

        // Normalize keywords (lowercase for matching)
        let normalized_keywords: Vec<String> = all_keywords
            .iter()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();

        println!(
            "{} Searching for {} keyword(s) using {} logic:",
            "•".cyan(),
            normalized_keywords.len(),
            logic.to_string().yellow().bold()
        );
        for kw in &normalized_keywords {
            println!("  {} {}", "→".green(), kw.white().bold());
        }
        println!();

        // Create output directory
        let output_dir = PathBuf::from("0_out");
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir)
                .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;
            println!(
                "{} Created output directory: {}",
                "•".cyan(),
                output_dir.display()
            );
        }

        // Add timestamp to filename (human-readable format)
        let timestamp = Local::now().format("%d%b%Y_%H%M%S");
        let checkpoint_file = output_dir.join(format!("{}_{}.csv", output_name, timestamp));
        let checkpoint_manager = CheckpointManager::new(&checkpoint_file, resume)?;

        Ok(Self {
            keywords: normalized_keywords,
            data_dir,
            output_name,
            logic,
            resume,
            threshold,
            checkpoint_manager,
            output_dir,
        })
    }

    /// Create runner from existing checkpoint file
    pub fn from_checkpoint(checkpoint_file: &PathBuf) -> Result<Self> {
        let checkpoint_manager = CheckpointManager::load(checkpoint_file)?;

        Ok(Self {
            keywords: Vec::new(),
            data_dir: PathBuf::from("."),
            output_name: checkpoint_file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            logic: KeywordLogic::And,
            resume: false,
            threshold: 1000,
            checkpoint_manager,
            output_dir: checkpoint_file
                .parent()
                .unwrap_or(Path::new("0_out"))
                .to_path_buf(),
        })
    }

    /// Get reference to checkpoint manager
    pub fn checkpoint_manager(&self) -> &CheckpointManager {
        &self.checkpoint_manager
    }

    /// Run the main search process
    pub fn run(&self) -> Result<()> {
        println!("{}", "═".repeat(80).cyan());
        println!(
            "{} {}",
            "Starting article search...".green().bold(),
            "📚".to_string()
        );
        println!("{}\n", "═".repeat(80).cyan());

        // Find all XML files
        let xml_files = self.find_xml_files()?;
        if xml_files.is_empty() {
            anyhow::bail!("No XML files found in directory: {:?}", self.data_dir);
        }

        println!(
            "{} Found {} XML file(s) in {}",
            "•".cyan(),
            xml_files.len(),
            self.data_dir.display()
        );

        // Count total articles
        let multi_progress = MultiProgress::new();
        let count_pb = multi_progress.add(ProgressBar::new(xml_files.len() as u64));
        count_pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} files | {msg}")
                .unwrap()
                .progress_chars("█▓▒░  "),
        );
        count_pb.set_message("Counting articles...");

        let total_articles = Arc::new(AtomicUsize::new(0));
        for xml_file in &xml_files {
            match count_articles(xml_file) {
                Ok(count) => {
                    total_articles.fetch_add(count, Ordering::SeqCst);
                }
                Err(e) => {
                    warn!("Error counting articles in {:?}: {}", xml_file, e);
                }
            }
            count_pb.inc(1);
        }
        count_pb.finish_with_message("Counting complete");

        let total = total_articles.load(Ordering::SeqCst);
        println!(
            "{} Total articles to process: {}\n",
            "•".cyan(),
            total.to_string().yellow().bold()
        );

        // Process articles
        let pb = multi_progress.add(ProgressBar::new(total as u64));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} | {per_sec} | {msg}")
                .unwrap()
                .progress_chars("█▓▒░  "),
        );

        let matched_count = Arc::new(AtomicUsize::new(0));

        for xml_file in &xml_files {
            self.process_xml_file(xml_file, &pb, &matched_count)?;
        }

        pb.finish_with_message("Processing complete");

        let final_matched = matched_count.load(Ordering::SeqCst);
        println!("\n{}", "═".repeat(80).cyan());
        println!(
            "{} {} articles matched your keywords!",
            "✓".green().bold(),
            final_matched.to_string().yellow().bold()
        );
        println!("{}\n", "═".repeat(80).cyan());

        // Warn if too many results
        if final_matched > self.threshold {
            println!(
                "{} {}",
                "⚠".yellow().bold(),
                format!(
                    "Warning: {} articles exceed threshold of {}",
                    final_matched, self.threshold
                )
                .yellow()
            );
            println!(
                "  {} Consider using more specific keywords for better model quality\n",
                "→".yellow()
            );
        }

        // Save checkpoint
        let checkpoint_path = self.checkpoint_manager.file_path();
        println!(
            "{} Saving results to {}",
            "•".cyan(),
            checkpoint_path.display().to_string().white().bold()
        );
        self.checkpoint_manager.finalize()?;

        // Generate statistics
        println!("{} Generating statistics...", "•".cyan());
        self.generate_statistics()?;

        println!(
            "\n{} {}",
            "✓".green().bold(),
            "Search complete!".green().bold()
        );
        println!("\n{}", "Next steps:".yellow().bold());
        let checkpoint_path = self.checkpoint_manager.file_path();
        println!(
            "  {} Review statistics in: {}",
            "1.".cyan(),
            self.output_dir.display().to_string().white()
        );
        println!(
            "  {} View visualizations: {}",
            "2.".cyan(),
            "python3 0_out/*_visualize.py".white()
        );
        println!(
            "  {} Generate model: {}",
            "3.".cyan(),
            format!(
                "curalit generate -c {} -m my-model",
                checkpoint_path.display()
            )
            .white()
        );

        // RAG workflow options
        println!(
            "\n{}",
            "RAG (Retrieval-Augmented Generation) Options:"
                .yellow()
                .bold()
        );
        println!(
            "  {} Automated RAG setup: {}",
            "•".cyan(),
            format!("./rag_workflow.sh {}", checkpoint_path.display()).white()
        );
        println!(
            "  {} Manual RAG build: {}",
            "•".cyan(),
            format!("curalit rag-build -c {}", checkpoint_path.display()).white()
        );
        println!(
            "\n  {} {}",
            "ℹ".blue(),
            "RAG retrieves exact facts from articles (no hallucination)".dimmed()
        );
        println!(
            "  {} {}",
            "ℹ".blue(),
            "Requires: Qdrant + Ollama with nomic-embed-text model".dimmed()
        );

        Ok(())
    }

    /// Find all XML files in data directory
    fn find_xml_files(&self) -> Result<Vec<PathBuf>> {
        let mut xml_files = Vec::new();

        for entry in WalkDir::new(&self.data_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "xml" {
                        xml_files.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(xml_files)
    }

    /// Process a single XML file
    fn process_xml_file(
        &self,
        xml_file: &PathBuf,
        pb: &ProgressBar,
        matched_count: &Arc<AtomicUsize>,
    ) -> Result<()> {
        let mut parser = PubMedParser::new(xml_file)?;

        while let Some(mut article) = parser.next_article()? {
            pb.inc(1);

            // Check if article matches keywords
            let matches = match self.logic {
                KeywordLogic::And => article.matches_all_keywords(&self.keywords),
                KeywordLogic::Or => article.matches_any_keyword(&self.keywords),
            };

            if matches {
                // Populate the keywords field with the matched search terms
                article.set_matched_keywords(&self.keywords);
                
                matched_count.fetch_add(1, Ordering::SeqCst);
                let count = matched_count.load(Ordering::SeqCst);
                pb.set_message(format!("{} matched", count));
                self.checkpoint_manager.add_article(&article)?;
            }
        }

        Ok(())
    }

    /// Generate statistics and visualizations
    pub fn generate_statistics(&self) -> Result<()> {
        let articles = self.checkpoint_manager.load_articles()?;

        let analyzer = StatisticsAnalyzer::new(articles, self.keywords.clone());
        let stats = analyzer.analyze()?;

        // Create timestamped filenames (human-readable format)
        let timestamp = Local::now().format("%d%b%Y_%H%M%S");

        // Save statistics to JSON
        let stats_file = self
            .output_dir
            .join(format!("{}_{}_stats.json", self.output_name, timestamp));
        let stats_json = serde_json::to_string_pretty(&stats)?;
        fs::write(&stats_file, stats_json)?;
        info!("Statistics saved to {}", stats_file.display());

        // Generate log file
        let log_file = self
            .output_dir
            .join(format!("{}_{}_stats.log", self.output_name, timestamp));
        stats.write_log(log_file.to_str().unwrap())?;
        println!(
            "  {} Statistics log: {}",
            "→".cyan(),
            log_file.display().to_string().white()
        );

        // Generate Python visualization script
        let viz_generator = VisualizationGenerator::new(
            stats,
            &self.output_name,
            &self.output_dir,
            &timestamp.to_string(),
        );
        viz_generator.generate()?;
        let viz_file = self
            .output_dir
            .join(format!("{}_{}_visualize.py", self.output_name, timestamp));
        println!(
            "  {} Visualization script: {}",
            "→".cyan(),
            viz_file.display().to_string().white()
        );

        Ok(())
    }

    /// Generate Ollama Modelfile
    pub fn generate_modelfile(&self, model_name: &str, base_model: &str) -> Result<()> {
        println!("{}", "═".repeat(80).cyan());
        println!(
            "{} {}",
            "Generating Ollama Modelfile...".green().bold(),
            "🤖"
        );
        println!("{}\n", "═".repeat(80).cyan());

        let articles = self.checkpoint_manager.load_articles()?;
        println!(
            "{} Loaded {} articles from checkpoint",
            "•".cyan(),
            articles.len()
        );

        let timestamp = Local::now().format("%d%b%Y_%H%M%S");
        let generator = ModelfileGenerator::new(model_name.to_string(), base_model.to_string());
        generator.generate(
            &articles,
            &self.output_name,
            &self.output_dir,
            &timestamp.to_string(),
        )?;

        let modelfile_path = self
            .output_dir
            .join(format!("Modelfile_{}_{}", model_name, timestamp));
        println!(
            "\n{} {}",
            "✓".green().bold(),
            "Modelfile generation complete!".green().bold()
        );
        println!("\n{}", "Next steps:".yellow().bold());
        println!(
            "  {} Create model: {}",
            "1.".cyan(),
            format!(
                "ollama create {} -f {}",
                model_name,
                modelfile_path.display()
            )
            .white()
        );
        println!(
            "  {} Run model: {}",
            "2.".cyan(),
            format!("ollama run {}", model_name).white()
        );

        Ok(())
    }

    /// Create runner from output directory (for packaging command)
    pub fn from_output_dir(output_dir: &PathBuf) -> Result<Self> {
        Ok(Self {
            keywords: Vec::new(),
            data_dir: PathBuf::from("."),
            output_name: String::from("model"),
            logic: KeywordLogic::And,
            resume: false,
            threshold: 1000,
            checkpoint_manager: CheckpointManager::new(&PathBuf::from("dummy.csv"), false)?,
            output_dir: output_dir.clone(),
        })
    }

    /// Package model files for distribution (called after generate)
    pub fn package_model(&self, model_name: &str, format: crate::cli::PackageFormat) -> Result<()> {
        use crate::cli::PackageFormat;

        println!("\n{}", "═".repeat(80).cyan());
        println!(
            "{} {}",
            "Packaging model for distribution...".green().bold(),
            "📦"
        );
        println!("{}\n", "═".repeat(80).cyan());

        // Find the most recent files for this model
        let model_files = self.find_model_files(model_name)?;

        if model_files.is_empty() {
            anyhow::bail!(
                "No model files found for '{}' in {:?}",
                model_name,
                self.output_dir
            );
        }

        println!(
            "{} Found {} file(s) to package:",
            "•".cyan(),
            model_files.len()
        );
        for file in &model_files {
            println!("  {} {}", "→".green(), file.display());
        }

        // Create package
        let package_name = format!("{}_distributable", model_name);
        let package_path = match format {
            PackageFormat::Tar => {
                let path = self.output_dir.join(format!("{}.tar.gz", package_name));
                self.create_tar_package(&model_files, &path)?;
                path
            }
            PackageFormat::Zip => {
                let path = self.output_dir.join(format!("{}.zip", package_name));
                self.create_zip_package(&model_files, &path)?;
                path
            }
        };

        println!(
            "\n{} {}",
            "✓".green().bold(),
            "Package created successfully!".green().bold()
        );
        println!(
            "  {} {}",
            "→".cyan(),
            package_path.display().to_string().white().bold()
        );

        println!("\n{}", "Distribution Instructions:".yellow().bold());
        println!("  {} Share the package file with others", "1.".cyan());
        println!("  {} Recipients should extract the archive", "2.".cyan());
        println!(
            "  {} Run: {}",
            "3.".cyan(),
            format!("ollama create {} -f Modelfile_*", model_name).white()
        );
        println!(
            "  {} Run: {}",
            "4.".cyan(),
            format!("ollama run {}", model_name).white()
        );

        Ok(())
    }

    /// Package model files with custom options (for package command)
    pub fn package_model_with_options(
        &self,
        model_name: &str,
        output_dir: &PathBuf,
        format: crate::cli::PackageFormat,
        output_name: Option<String>,
    ) -> Result<()> {
        use crate::cli::PackageFormat;

        println!("{}", "═".repeat(80).cyan());
        println!(
            "{} {}",
            "Packaging model for distribution...".green().bold(),
            "📦"
        );
        println!("{}\n", "═".repeat(80).cyan());

        // Find model files
        let model_files = self.find_model_files_in_dir(model_name, output_dir)?;

        if model_files.is_empty() {
            anyhow::bail!(
                "No model files found for '{}' in {:?}",
                model_name,
                output_dir
            );
        }

        println!(
            "{} Found {} file(s) to package:",
            "•".cyan(),
            model_files.len()
        );
        for file in &model_files {
            println!("  {} {}", "→".green(), file.display());
        }

        // Determine package name
        let package_name = output_name.unwrap_or_else(|| format!("{}_distributable", model_name));

        // Ensure output directory exists
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)
                .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;
        }

        // Create package
        let package_path = match format {
            PackageFormat::Tar => {
                let path = output_dir.join(format!("{}.tar.gz", package_name));
                self.create_tar_package(&model_files, &path)?;
                path
            }
            PackageFormat::Zip => {
                let path = output_dir.join(format!("{}.zip", package_name));
                self.create_zip_package(&model_files, &path)?;
                path
            }
        };

        println!(
            "\n{} {}",
            "✓".green().bold(),
            "Package created successfully!".green().bold()
        );
        println!(
            "  {} {}",
            "→".cyan(),
            package_path.display().to_string().white().bold()
        );

        Ok(())
    }

    /// Find all files related to a model
    fn find_model_files(&self, model_name: &str) -> Result<Vec<PathBuf>> {
        self.find_model_files_in_dir(model_name, &self.output_dir)
    }

    /// Find all files related to a model in a specific directory
    fn find_model_files_in_dir(&self, model_name: &str, dir: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Look for Modelfile, training data, and system prompt
        for entry in WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let filename = path.file_name().unwrap_or_default().to_string_lossy();

            // Match files containing the model name
            if filename.contains(&format!("Modelfile_{}", model_name))
                || filename.contains("_training.jsonl")
                || filename.contains("_system_prompt.txt")
            {
                files.push(path.to_path_buf());
            }
        }

        // Sort by modification time (most recent first) and take the most recent set
        files.sort_by(|a, b| {
            let time_a = fs::metadata(a).and_then(|m| m.modified()).ok();
            let time_b = fs::metadata(b).and_then(|m| m.modified()).ok();
            time_b.cmp(&time_a)
        });

        // Take the most recent Modelfile and associated files
        if let Some(modelfile) = files.iter().find(|f| {
            f.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with("Modelfile_")
        }) {
            // Extract timestamp from Modelfile
            let modelfile_name = modelfile.file_name().unwrap_or_default().to_string_lossy();
            if let Some(timestamp) = modelfile_name.split('_').last() {
                // Filter files with matching timestamp
                let matching_files: Vec<PathBuf> = files
                    .iter()
                    .filter(|f| {
                        f.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .contains(timestamp)
                    })
                    .cloned()
                    .collect();

                if !matching_files.is_empty() {
                    return Ok(matching_files);
                }
            }
        }

        // Fallback: return first 3 files (Modelfile, training, prompt)
        Ok(files.into_iter().take(3).collect())
    }

    /// Create tar.gz package
    fn create_tar_package(&self, files: &[PathBuf], output_path: &PathBuf) -> Result<()> {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let tar_gz = fs::File::create(output_path)
            .with_context(|| format!("Failed to create tar.gz file: {:?}", output_path))?;
        let encoder = GzEncoder::new(tar_gz, Compression::default());
        let mut archive = tar::Builder::new(encoder);

        for file in files {
            let filename = file.file_name().unwrap_or_default();
            archive
                .append_path_with_name(file, filename)
                .with_context(|| format!("Failed to add file to archive: {:?}", file))?;
        }

        // Add README for distribution
        let readme_content = self.create_distribution_readme();
        let mut header = tar::Header::new_gnu();
        header.set_size(readme_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append_data(
            &mut header,
            "README_DISTRIBUTION.md",
            readme_content.as_bytes(),
        )?;

        archive.finish()?;
        Ok(())
    }

    /// Create zip package
    fn create_zip_package(&self, files: &[PathBuf], output_path: &PathBuf) -> Result<()> {
        use std::io::Write;
        use zip::write::FileOptions;

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let file = fs::File::create(output_path)
            .with_context(|| format!("Failed to create zip file: {:?}", output_path))?;
        let mut zip = zip::ZipWriter::new(file);

        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        for file_path in files {
            let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
            zip.start_file(filename.to_string(), options)?;

            let content = fs::read(file_path)
                .with_context(|| format!("Failed to read file: {:?}", file_path))?;
            zip.write_all(&content)?;
        }

        // Add README for distribution
        let readme_content = self.create_distribution_readme();
        zip.start_file("README_DISTRIBUTION.md", options)?;
        zip.write_all(readme_content.as_bytes())?;

        zip.finish()?;
        Ok(())
    }

    /// Create README for distribution package
    fn create_distribution_readme(&self) -> String {
        format!(
            r#"# CuraLit Model Distribution Package

This package contains a custom Ollama model generated by CuraLit from curated PubMed literature.

## Contents

- **Modelfile_***: Ollama configuration file for the model
- ***_training.jsonl**: Training data in JSONL format (for reference)
- ***_system_prompt.txt**: Custom system prompt (for reference)

## Installation Instructions

### Prerequisites

1. Install Ollama: https://ollama.ai/download
2. Verify installation: `ollama --version`

### Creating the Model

1. Extract this archive to a directory
2. Navigate to the extracted directory
3. Create the model:
   ```bash
   ollama create <model-name> -f Modelfile_*
   ```
   Replace `<model-name>` with your desired model name

4. Verify the model was created:
   ```bash
   ollama list
   ```

### Running the Model

Start an interactive session:
```bash
ollama run <model-name>
```

### Using the Model

The model is trained on specific biomedical literature from PubMed. It can:
- Answer questions about articles in its training corpus
- Explain biomedical concepts for novices and experts
- Help with hypothesis generation and research planning
- Assist with literature review synthesis
- Provide PMID citations

### Example Queries

- "What are the main findings about [topic] in your training corpus?"
- "Explain [concept] in simple terms"
- "What research gaps exist in [area]?"
- "Compare the methodologies used in studies about [topic]"

## Notes

- The model's knowledge is limited to the articles in the training corpus
- Training data and system prompt files are included for reference
- You can customize the Modelfile before creating the model if needed

## Support

For issues or questions about CuraLit, visit:
https://github.com/yourusername/curalit

Generated by CuraLit v0.2.1
"#
        )
    }
}

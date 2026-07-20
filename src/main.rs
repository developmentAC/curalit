use anyhow::Result;
use clap::Parser;
use colored::*;
use curalit::cli::{Cli, Commands};
use curalit::rag::{RagConfig, RagSystem};
use curalit::runner::CuraLitRunner;
use env_logger::Env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    // Print banner
    print_banner();

    // Parse CLI arguments
    let cli = Cli::parse();

    match cli.command {
        Commands::BigHelp => {
            print_big_help();
            Ok(())
        }
        Commands::Search {
            keywords,
            keywords_file,
            data_dir,
            output,
            logic,
            resume,
            threshold,
        } => {
            let output_name = output.clone(); // Clone before move
            let runner = CuraLitRunner::new(
                keywords,
                keywords_file,
                data_dir,
                output,
                logic,
                resume,
                threshold,
            )?;
            runner.run()?;

            // Print next steps after search completes
            println!("\n{}", "═".repeat(80).cyan());
            println!(
                "{} {}",
                "✓".green().bold(),
                "Search complete!".green().bold()
            );
            println!("\n{}", "Next steps:".yellow().bold());
            println!(
                "  {} Review statistics: {}",
                "1.".cyan(),
                format!("curalit stats -c 0_out/{}_*.csv", output_name).white()
            );
            println!(
                "  {} Build RAG index (recommended): {}",
                "2.".cyan(),
                format!("curalit rag-build -c 0_out/{}_*.csv", output_name).white()
            );
            println!(
                "  {} Build verification database: {}",
                "3.".cyan(),
                "curalit db-build -k \"keyword1\" -k \"keyword2\" -d ./data".white()
            );
            println!("\n{}", "RAG Workflow (Recommended):".yellow().bold());
            println!(
                "  {} Query with citations: {}",
                "→".cyan(),
                "curalit rag-generate -q \"your question\" -m llama3".white()
            );
            println!(
                "  {} Verify citations: {}",
                "→".cyan(),
                "curalit rag-generate -q \"question\" -m llama3 --use-db 0_out/database.db".white()
            );
            println!("\n{}", "Traditional Workflow:".yellow().bold());
            println!(
                "  {} Generate model: {}",
                "→".cyan(),
                format!(
                    "curalit generate -c 0_out/{}_*.csv -m my-model",
                    output_name
                )
                .white()
            );
            println!("{}", "═".repeat(80).cyan());

            Ok(())
        }
        Commands::Stats { checkpoint_file } => {
            let runner = CuraLitRunner::from_checkpoint(&checkpoint_file)?;
            runner.generate_statistics()?;

            // Print next steps after statistics generation
            println!("\n{}", "═".repeat(80).cyan());
            println!(
                "{} {}",
                "✓".green().bold(),
                "Statistics generated successfully!".green().bold()
            );
            println!("\n{}", "Next steps:".yellow().bold());
            println!(
                "  {} Review statistics: {}",
                "1.".cyan(),
                "0_out/*_stats.log".white()
            );
            println!(
                "  {} View visualizations: {}",
                "2.".cyan(),
                "python 0_out/*_visualize.py".white()
            );
            println!(
                "\n{}",
                "RAG Workflow (Recommended for accuracy):".yellow().bold()
            );
            println!(
                "  {} Build RAG index: {}",
                "3.".cyan(),
                format!("curalit rag-build -c {}", checkpoint_file.display()).white()
            );
            println!(
                "  {} Build verification database: {}",
                "4.".cyan(),
                "curalit db-build -k \"keyword1\" -k \"keyword2\" -d ./data".white()
            );
            println!(
                "  {} Query with verified citations: {}",
                "5.".cyan(),
                "curalit rag-generate -q \"question\" -m llama3 --use-db 0_out/database.db".white()
            );
            println!("\n{}", "Traditional Fine-tuning:".yellow().bold());
            println!(
                "  {} Generate model: {}",
                "6.".cyan(),
                format!(
                    "curalit generate -c {} -m my-model",
                    checkpoint_file.display()
                )
                .white()
            );
            println!("{}", "═".repeat(80).cyan());

            Ok(())
        }
        Commands::Generate {
            checkpoint_file,
            model_name,
            base_model,
            package,
            package_format,
        } => {
            let runner = CuraLitRunner::from_checkpoint(&checkpoint_file)?;
            runner.generate_modelfile(&model_name, &base_model)?;

            // Optionally create distributable package
            if package {
                let format = package_format.unwrap_or(curalit::cli::PackageFormat::Tar);
                runner.package_model(&model_name, format)?;
            }
            Ok(())
        }
        Commands::Package {
            model_name,
            output_dir,
            format,
            output_name,
        } => {
            let runner = CuraLitRunner::from_output_dir(&output_dir)?;
            runner.package_model_with_options(&model_name, &output_dir, format, output_name)
        }
        Commands::RagBuild {
            checkpoint_file,
            embedding_model,
            storage_path,
            collection_name,
        } => {
            // Load articles from checkpoint
            let runner = CuraLitRunner::from_checkpoint(&checkpoint_file)?;
            let checkpoint_manager = runner.checkpoint_manager();
            let articles = checkpoint_manager.load_articles()?;

            // Create RAG config
            let config = RagConfig {
                ollama_url: "http://localhost:11434".to_string(),
                embedding_model,
                collection_name,
                qdrant_path: storage_path,
                chunk_size: 500,
                chunk_overlap: 50,
                top_k: 5,
            };

            // Build RAG index
            let mut rag = RagSystem::with_config(config.clone());
            rag.initialize().await?;
            rag.build_index(&articles).await?;

            // Save config
            let config_path = rag.save_config(&PathBuf::from("0_out"))?;
            println!(
                "\n{} Saved RAG config to: {}",
                "•".cyan(),
                config_path.display()
            );

            // Print next steps
            println!("\n{}", "═".repeat(80).cyan());
            println!(
                "{} {}",
                "✓".green().bold(),
                "RAG index built successfully!".green().bold()
            );
            println!("\n{}", "Next steps:".yellow().bold());
            println!(
                "  {} Query the index: {}",
                "1.".cyan(),
                format!("curalit rag-query -q \"your question\"").white()
            );
            println!(
                "  {} Generate answers: {}",
                "2.".cyan(),
                format!("curalit rag-generate -q \"your question\" -m llama3").white()
            );
            println!(
                "  {} Package for sharing: {}",
                "3.".cyan(),
                format!("curalit rag-package -n {}", config.collection_name).white()
            );
            println!(
                "\n{}",
                "  For verified citations (prevent hallucination):"
                    .yellow()
                    .bold()
            );
            println!(
                "  {} Build database: {}",
                "4.".cyan(),
                "curalit db-build -k \"keyword1\" -k \"keyword2\" -d ./data".white()
            );
            println!(
                "  {} Use with RAG: {}",
                "5.".cyan(),
                "curalit rag-generate -q \"question\" -m llama3 --use-db 0_out/database.db".white()
            );
            println!("{}", "═".repeat(80).cyan());

            Ok(())
        }
        Commands::RagQuery {
            query,
            storage_path,
            collection_name,
            embedding_model,
            top_k,
        } => {
            // Create RAG config
            let config = RagConfig {
                ollama_url: "http://localhost:11434".to_string(),
                embedding_model,
                collection_name,
                qdrant_path: storage_path,
                chunk_size: 500,
                chunk_overlap: 50,
                top_k,
            };

            // Query RAG index
            let mut rag = RagSystem::with_config(config);
            rag.initialize().await?;

            println!("{} Searching for: {}", "•".cyan(), query.white().bold());
            let chunks = rag.query(&query).await?;

            println!("\n{}", "═".repeat(80).cyan());
            println!(
                "{} Found {} relevant passages:\n",
                "✓".green().bold(),
                chunks.len()
            );

            for (idx, chunk) in chunks.iter().enumerate() {
                println!("{}", format!("\n[Result {}]", idx + 1).yellow().bold());
                println!("{} {}", "PMID:".cyan(), chunk.pmid.white());
                println!("{} {}", "Title:".cyan(), chunk.title);
                println!("{} {}", "Authors:".cyan(), chunk.authors.join(", "));
                println!("{} {}", "Journal:".cyan(), chunk.journal);
                println!("{}\n{}", "Text:".cyan(), chunk.text);
                println!("{}", "─".repeat(80).dimmed());
            }

            // Print next steps
            println!("\n{}", "═".repeat(80).cyan());
            println!(
                "{} {}",
                "✓".green().bold(),
                "Query complete!".green().bold()
            );
            println!("\n{}", "Next steps:".yellow().bold());
            println!(
                "  {} Generate full answer: {}",
                "1.".cyan(),
                format!("curalit rag-generate -q \"{}\" -m llama3", query).white()
            );
            println!(
                "  {} Verify with database: {}",
                "2.".cyan(),
                format!(
                    "curalit rag-generate -q \"{}\" -m llama3 --use-db 0_out/database.db",
                    query
                )
                .white()
            );
            println!(
                "  {} Try different query: {}",
                "3.".cyan(),
                "curalit rag-query -q \"new question\"".white()
            );
            println!("{}", "═".repeat(80).cyan());

            Ok(())
        }
        Commands::RagGenerate {
            query,
            model,
            storage_path,
            collection_name,
            embedding_model,
            top_k,
            use_db,
        } => {
            // Create RAG config
            let config = RagConfig {
                ollama_url: "http://localhost:11434".to_string(),
                embedding_model,
                collection_name,
                qdrant_path: storage_path,
                chunk_size: 500,
                chunk_overlap: 50,
                top_k,
            };

            // Generate answer using RAG
            let mut rag = RagSystem::with_config(config);
            rag.initialize().await?;

            println!("{}", "═".repeat(80).cyan());
            println!(
                "{} {}",
                "RAG-Powered Answer Generation".green().bold(),
                "🤖"
            );
            println!("{}\n", "═".repeat(80).cyan());
            println!("{} {}\n", "Question:".yellow().bold(), query.white());

            let answer = rag.generate_answer(&query, &model).await?;

            println!("\n{}", "═".repeat(80).cyan());
            println!("{}\n", "Answer:".yellow().bold());
            println!("{}\n", answer);

            // If database verification is enabled, extract and verify factual information
            if let Some(ref db_path) = use_db {
                use curalit::database::ArticleDatabase;
                use regex::Regex;

                println!("{}", "═".repeat(80).cyan());
                println!(
                    "{}",
                    "Database Verification (PMID/DOI Fact-Checking)"
                        .yellow()
                        .bold()
                );
                println!("{}\n", "═".repeat(80).cyan());

                let db = ArticleDatabase::open(&db_path)?;

                // Extract PMIDs from the answer using regex
                let pmid_regex = Regex::new(r"\b(?:PMID:?\s*)?(\d{7,8})\b")?;
                let mut verified_info = Vec::new();

                for cap in pmid_regex.captures_iter(&answer) {
                    if let Some(pmid) = cap.get(1) {
                        let pmid_str = pmid.as_str();
                        if let Some(article_info) = db.get_by_pmid(pmid_str)? {
                            verified_info.push(article_info);
                            println!("{} Verified PMID: {}", "✓".green(), pmid_str);
                        } else {
                            println!("{} PMID not found in database: {}", "⚠".yellow(), pmid_str);
                        }
                    }
                }

                if !verified_info.is_empty() {
                    println!("\n{}", "Verified Citations:".cyan().bold());
                    for info in verified_info {
                        println!("\n{}", "─".repeat(80).dimmed());
                        println!("{}", info.format_citation());
                    }
                    println!("\n{}", "─".repeat(80).dimmed());
                } else {
                    println!("\n{} No PMIDs found in the answer to verify.", "ℹ".cyan());
                }
            }

            println!("{}", "═".repeat(80).cyan());

            // Print next steps
            println!("\n{}", "Next steps:".yellow().bold());
            println!(
                "  {} Query again: {}",
                "1.".cyan(),
                "curalit rag-generate -q \"new question\" -m llama3".white()
            );
            if use_db.is_none() {
                println!(
                    "  {} Add verification: {}",
                    "2.".cyan(),
                    "Build database with 'curalit db-build' and use --use-db flag".white()
                );
            }
            println!(
                "  {} Export results: {}",
                "3.".cyan(),
                "Save answer to file for documentation".white()
            );
            println!("{}", "═".repeat(80).cyan());

            Ok(())
        }
        Commands::RagPackage {
            collection_name,
            storage_path,
            output_name,
            format,
            output_dir,
        } => {
            use colored::*;
            use std::fs;
            use std::path::Path;

            println!("{}", "═".repeat(80).cyan());
            println!(
                "{} {}",
                "Packaging RAG model for distribution...".green().bold(),
                "📦"
            );
            println!("{}\n", "═".repeat(80).cyan());

            // Verify storage path exists
            if !storage_path.exists() {
                anyhow::bail!(
                    "Qdrant storage path not found: {:?}\nHave you built a RAG index with 'curalit rag-build'?",
                    storage_path
                );
            }

            // Check for the collection
            let collection_path = storage_path.join("collections").join(&collection_name);
            if !collection_path.exists() {
                anyhow::bail!(
                    "Collection '{}' not found in storage path.\nAvailable collections: {:?}",
                    collection_name,
                    fs::read_dir(storage_path.join("collections"))
                        .ok()
                        .map(|entries| {
                            entries
                                .filter_map(|e| e.ok())
                                .map(|e| e.file_name().to_string_lossy().to_string())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                );
            }

            println!("{} Collection '{}' found", "✓".green(), collection_name);
            println!("{} Storage path: {}", "•".cyan(), storage_path.display());

            // Create output directory
            if !output_dir.exists() {
                fs::create_dir_all(&output_dir)?;
            }

            // Determine package name
            let package_name =
                output_name.unwrap_or_else(|| format!("{}_rag_model", collection_name));

            // Create package
            let package_path = match format {
                curalit::cli::PackageFormat::Tar => {
                    use flate2::write::GzEncoder;
                    use flate2::Compression;

                    let path = output_dir.join(format!("{}.tar.gz", package_name));
                    let tar_gz = fs::File::create(&path)?;
                    let encoder = GzEncoder::new(tar_gz, Compression::default());
                    let mut archive = tar::Builder::new(encoder);

                    // Add entire qdrant_storage directory
                    println!("{} Adding vector database to archive...", "•".cyan());
                    archive.append_dir_all("qdrant_storage", &storage_path)?;

                    // Add RAG config if it exists
                    let config_path = PathBuf::from("0_out/rag_config.json");
                    if config_path.exists() {
                        println!("{} Adding RAG configuration...", "•".cyan());
                        archive.append_path_with_name(&config_path, "rag_config.json")?;
                    }

                    // Create and add setup script
                    println!("{} Creating setup script...", "•".cyan());
                    let setup_script = create_rag_setup_script(&collection_name);
                    let mut header = tar::Header::new_gnu();
                    header.set_size(setup_script.len() as u64);
                    header.set_mode(0o755); // Make executable
                    header.set_cksum();
                    archive.append_data(&mut header, "setup_rag.sh", setup_script.as_bytes())?;

                    // Add README
                    println!("{} Creating README...", "•".cyan());
                    let readme = create_rag_distribution_readme(&collection_name);
                    let mut header = tar::Header::new_gnu();
                    header.set_size(readme.len() as u64);
                    header.set_mode(0o644);
                    header.set_cksum();
                    archive.append_data(&mut header, "README_RAG.md", readme.as_bytes())?;

                    archive.finish()?;
                    path
                }
                curalit::cli::PackageFormat::Zip => {
                    use std::io::Write;
                    use walkdir::WalkDir;
                    use zip::write::FileOptions;

                    let path = output_dir.join(format!("{}.zip", package_name));
                    let file = fs::File::create(&path)?;
                    let mut zip = zip::ZipWriter::new(file);
                    let options = FileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated)
                        .unix_permissions(0o644);

                    // Add entire qdrant_storage directory
                    println!("{} Adding vector database to archive...", "•".cyan());
                    for entry in WalkDir::new(&storage_path) {
                        let entry = entry?;
                        let path = entry.path();
                        let name = path
                            .strip_prefix(&storage_path.parent().unwrap_or(Path::new(".")))
                            .unwrap();

                        if path.is_file() {
                            zip.start_file(name.to_string_lossy().to_string(), options)?;
                            let content = fs::read(path)?;
                            zip.write_all(&content)?;
                        } else if path.is_dir() && !name.as_os_str().is_empty() {
                            zip.add_directory(name.to_string_lossy().to_string(), options)?;
                        }
                    }

                    // Add RAG config if it exists
                    let config_path = PathBuf::from("0_out/rag_config.json");
                    if config_path.exists() {
                        println!("{} Adding RAG configuration...", "•".cyan());
                        zip.start_file("rag_config.json", options)?;
                        let content = fs::read(&config_path)?;
                        zip.write_all(&content)?;
                    }

                    // Add setup script
                    println!("{} Creating setup script...", "•".cyan());
                    let setup_script = create_rag_setup_script(&collection_name);
                    let exec_options = FileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated)
                        .unix_permissions(0o755);
                    zip.start_file("setup_rag.sh", exec_options)?;
                    zip.write_all(setup_script.as_bytes())?;

                    // Add README
                    println!("{} Creating README...", "•".cyan());
                    let readme = create_rag_distribution_readme(&collection_name);
                    zip.start_file("README_RAG.md", options)?;
                    zip.write_all(readme.as_bytes())?;

                    zip.finish()?;
                    path
                }
            };

            println!(
                "\n{} {}",
                "✓".green().bold(),
                "RAG package created successfully!".green().bold()
            );
            println!(
                "  {} {}",
                "→".cyan(),
                package_path.display().to_string().white().bold()
            );

            // Get package size
            if let Ok(metadata) = fs::metadata(&package_path) {
                let size_mb = metadata.len() as f64 / 1_048_576.0;
                println!("  {} {:.2} MB", "Size:".cyan(), size_mb);
            }

            println!("\n{}", "Distribution Instructions:".yellow().bold());
            println!("  {} Share the package file with others", "1.".cyan());
            println!("  {} Recipients should extract the archive", "2.".cyan());
            println!(
                "  {} Run the setup script: {}",
                "3.".cyan(),
                "./setup_rag.sh".white()
            );
            println!(
                "  {} Query the knowledge base: {}",
                "4.".cyan(),
                format!(
                    "curalit rag-query -q \"your question\" -n {}",
                    collection_name
                )
                .white()
            );
            println!(
                "  {} Generate answers: {}",
                "5.".cyan(),
                format!(
                    "curalit rag-generate -q \"your question\" -m llama3 -n {}",
                    collection_name
                )
                .white()
            );

            Ok(())
        }
        Commands::DbBuild {
            keywords,
            keywords_file,
            data_dir,
            output_dir,
            db_name,
            logic,
        } => {
            use curalit::database::ArticleDatabase;
            use curalit::parser::PubMedParser;
            use indicatif::{ProgressBar, ProgressStyle};
            use std::fs;
            use walkdir::WalkDir;

            println!("{}", "═".repeat(80).cyan());
            println!(
                "{} {}",
                "Building SQLite Database for Fact Verification"
                    .green()
                    .bold(),
                "🗄️"
            );
            println!("{}\n", "═".repeat(80).cyan());

            // Load keywords
            let mut all_keywords = keywords;
            if let Some(kw_file) = keywords_file {
                let content = fs::read_to_string(&kw_file)?;
                all_keywords.extend(
                    content
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                );
            }

            if all_keywords.is_empty() {
                anyhow::bail!("No keywords provided. Use -k or -f to specify keywords.");
            }

            println!(
                "{} Keywords ({} logic): {}",
                "•".cyan(),
                logic.to_string().yellow(),
                all_keywords
                    .iter()
                    .map(|k| format!("\"{}\"", k))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!("{} Data directory: {}", "•".cyan(), data_dir.display());

            // Create output directory
            fs::create_dir_all(&output_dir)?;

            // Generate human-readable timestamp
            let timestamp = chrono::Local::now().format("%d%b%Y_%H%M%S");
            let db_path = output_dir.join(format!("{}_{}.db", db_name, timestamp));

            println!("{} Database: {}\n", "•".cyan(), db_path.display());

            // Create database
            let db = ArticleDatabase::create(&db_path)?;

            // Find XML files
            let xml_files: Vec<_> = WalkDir::new(&data_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("xml"))
                        .unwrap_or(false)
                })
                .map(|e| e.path().to_path_buf())
                .collect();

            if xml_files.is_empty() {
                anyhow::bail!("No XML files found in {}", data_dir.display());
            }

            println!("{} Found {} XML files\n", "✓".green(), xml_files.len());

            // Progress bar
            let pb = ProgressBar::new(xml_files.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
                    .progress_chars("█▓▒░"),
            );

            let mut total_articles = 0;
            let mut matched_articles = Vec::new();

            // Parse XML files
            for xml_file in &xml_files {
                pb.set_message(format!(
                    "Processing: {}",
                    xml_file.file_name().unwrap().to_string_lossy()
                ));

                let mut parser = PubMedParser::new(xml_file)?;

                while let Some(mut article) = parser.next_article()? {
                    total_articles += 1;

                    // Check if article matches keywords
                    let matches = match logic {
                        curalit::cli::KeywordLogic::And => {
                            article.matches_all_keywords(&all_keywords)
                        }
                        curalit::cli::KeywordLogic::Or => {
                            article.matches_any_keyword(&all_keywords)
                        }
                    };

                    if matches {
                        // Populate the keywords field with the matched search terms
                        article.set_matched_keywords(&all_keywords);
                        
                        matched_articles.push(article);

                        // Insert in batches of 1000 for performance
                        if matched_articles.len() >= 1000 {
                            db.insert_articles(&matched_articles)?;
                            matched_articles.clear();
                        }
                    }
                }

                pb.inc(1);
            }

            // Insert remaining articles
            if !matched_articles.is_empty() {
                db.insert_articles(&matched_articles)?;
            }

            pb.finish_with_message("Complete!");

            // Get database statistics
            let stats = db.get_stats()?;

            println!("\n{}", "═".repeat(80).cyan());
            println!("{}", "Database Statistics".yellow().bold());
            println!("{}", "═".repeat(80).cyan());
            println!(
                "{} Total articles processed: {}",
                "•".cyan(),
                total_articles
            );
            println!(
                "{} Matched articles in database: {}",
                "•".cyan(),
                stats.total_articles
            );
            println!("{} Articles with DOI: {}", "•".cyan(), stats.with_doi);
            println!(
                "{} Articles with abstract: {}",
                "•".cyan(),
                stats.with_abstract
            );
            println!("{}", "═".repeat(80).cyan());

            println!(
                "\n{} {}",
                "✓".green().bold(),
                "Database created successfully!".green().bold()
            );
            println!(
                "  {} {}",
                "→".cyan(),
                db_path.display().to_string().white().bold()
            );

            println!("\n{}", "Usage with RAG:".yellow().bold());
            println!(
                "  {}",
                format!(
                    "curalit rag-generate -q \"your question\" -m llama3 --use-db {}",
                    db_path.display()
                )
                .cyan()
            );

            Ok(())
        }
    }
}

fn create_rag_setup_script(collection_name: &str) -> String {
    format!(
        r#"#!/bin/bash
# RAG Model Setup Script
# This script sets up the Qdrant vector database for the RAG model

set -e

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${{CYAN}}╔═══════════════════════════════════════════════════════╗${{NC}}"
echo -e "${{CYAN}}║         CuraLit RAG Model Setup                       ║${{NC}}"
echo -e "${{CYAN}}╚═══════════════════════════════════════════════════════╝${{NC}}"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${{YELLOW}}Docker is not installed. Please install Docker first.${{NC}}"
    echo "Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

echo -e "${{GREEN}}✓ Docker is installed${{NC}}"

# Check if Qdrant container exists
if docker ps -a --format '{{{{.Names}}}}' | grep -q '^curalit-qdrant$'; then
    echo -e "${{YELLOW}}Stopping existing Qdrant container...${{NC}}"
    docker stop curalit-qdrant 2>/dev/null || true
    docker rm curalit-qdrant 2>/dev/null || true
fi

# Start Qdrant with the packaged database
echo -e "${{CYAN}}Starting Qdrant with packaged database...${{NC}}"
docker run -d \
    --name curalit-qdrant \
    -p 6333:6333 \
    -p 6334:6334 \
    -v "$(pwd)/qdrant_storage:/qdrant/storage" \
    qdrant/qdrant

# Wait for Qdrant to be ready
echo -e "${{CYAN}}Waiting for Qdrant to start...${{NC}}"
for i in {{1..30}}; do
    if curl -s http://localhost:6333/healthz > /dev/null 2>&1; then
        echo -e "${{GREEN}}✓ Qdrant is ready!${{NC}}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${{YELLOW}}Qdrant startup timeout. Check logs: docker logs curalit-qdrant${{NC}}"
        exit 1
    fi
    sleep 1
done

# Check Ollama
if ! command -v ollama &> /dev/null; then
    echo -e "${{YELLOW}}Ollama is not installed. Please install Ollama first.${{NC}}"
    echo "Visit: https://ollama.ai/"
    exit 1
fi

echo -e "${{GREEN}}✓ Ollama is installed${{NC}}"

# Check if Ollama is running
if ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo -e "${{YELLOW}}Ollama is not running. Please start Ollama.${{NC}}"
    exit 1
fi

echo -e "${{GREEN}}✓ Ollama is running${{NC}}"

# Pull embedding model
echo -e "${{CYAN}}Checking embedding model...${{NC}}"
if ! ollama list | grep -q "nomic-embed-text"; then
    echo -e "${{CYAN}}Pulling nomic-embed-text model...${{NC}}"
    ollama pull nomic-embed-text
fi

echo -e "${{GREEN}}✓ Embedding model ready${{NC}}"

echo ""
echo -e "${{GREEN}}╔═══════════════════════════════════════════════════════╗${{NC}}"
echo -e "${{GREEN}}║            Setup Complete!                            ║${{NC}}"
echo -e "${{GREEN}}╚═══════════════════════════════════════════════════════╝${{NC}}"
echo ""
echo "You can now query the RAG model:"
echo ""
echo -e "  Query passages:  ${{CYAN}}curalit rag-query -q \"your question\" -n {}${{NC}}"
echo -e "  Generate answer: ${{CYAN}}curalit rag-generate -q \"your question\" -m llama3 -n {}${{NC}}"
echo ""
"#,
        collection_name, collection_name
    )
}

fn create_rag_distribution_readme(collection_name: &str) -> String {
    format!(
        r#"# CuraLit RAG Model Distribution Package

This package contains a pre-built RAG (Retrieval-Augmented Generation) model with an embedded vector database.

## What's Included

- **qdrant_storage/**: Vector database containing embedded article content
- **rag_config.json**: Configuration for the RAG system
- **setup_rag.sh**: Automated setup script
- **README_RAG.md**: This file

## System Requirements

- **Docker**: For running Qdrant vector database
- **Ollama**: For embeddings and LLM generation
- **CuraLit**: The CuraLit CLI tool

### Installation

1. **Install Docker**  
   Visit: https://docs.docker.com/get-docker/

2. **Install Ollama**  
   Visit: https://ollama.ai/

3. **Install CuraLit**  
   ```bash
   # From source
   git clone <curalit-repo>
   cd curalit
   cargo install --path .
   ```

## Quick Start

1. **Extract this package**:
   ```bash
   tar -xzf {}_rag_model.tar.gz
   cd {}_rag_model/
   ```

2. **Run the setup script**:
   ```bash
   chmod +x setup_rag.sh
   ./setup_rag.sh
   ```

3. **Query the knowledge base**:
   ```bash
   # Get relevant passages
   curalit rag-query -q "What are the mechanisms?" -n {}

   # Generate complete answers with citations
   curalit rag-generate -q "Compare treatment approaches" -m llama3 -n {}
   ```

## Manual Setup (Alternative)

If the setup script doesn't work, follow these steps:

1. **Start Qdrant**:
   ```bash
   docker run -d \
       --name curalit-qdrant \
       -p 6333:6333 \
       -p 6334:6334 \
       -v "$(pwd)/qdrant_storage:/qdrant/storage" \
       qdrant/qdrant
   ```

2. **Install embedding model**:
   ```bash
   ollama pull nomic-embed-text
   ```

3. **Use the RAG system**:
   ```bash
   curalit rag-query -q "your question" -n {}
   curalit rag-generate -q "your question" -m llama3 -n {}
   ```

## Troubleshooting

### Qdrant Connection Issues

```bash
# Check if Qdrant is running
docker ps | grep qdrant

# View Qdrant logs
docker logs curalit-qdrant

# Restart Qdrant
docker restart curalit-qdrant
```

### Ollama Issues

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# List installed models
ollama list

# Re-pull embedding model
ollama pull nomic-embed-text
```

## About RAG Models

This RAG model provides:
- **Accurate citations**: Answers are backed by specific articles
- **No hallucination**: Responses are grounded in the embedded literature
- **No fine-tuning needed**: Use any LLM with the same knowledge base

The vector database contains chunked article content with embeddings, allowing for semantic search and retrieval.

## Collection Information

- **Collection Name**: {}
- **Model**: Optimized for biomedical/scientific queries
- **Embedding Model**: nomic-embed-text (via Ollama)

## Support

For issues or questions, please refer to the CuraLit documentation or repository.
"#,
        collection_name,
        collection_name,
        collection_name,
        collection_name,
        collection_name,
        collection_name,
        collection_name
    )
}

fn print_banner() {
    println!(
        "\n{}",
        "╔═══════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║                                                           ║".cyan()
    );
    println!(
        "{}",
        "║                     CuraLit v0.4.1                        ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "║           Literature-Driven LLM Generator                 ║".cyan()
    );
    println!(
        "{}",
        "║                                                           ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════╝".cyan()
    );
    println!();
}

fn print_big_help() {
    println!(
        "\n{}\n",
        "CuraLit - Comprehensive Help Guide".green().bold()
    );

    println!("{}", "═".repeat(80).cyan());
    println!("\n{}\n", "OVERVIEW".yellow().bold());
    println!("CuraLit extracts relevant articles from PubMed XML datasets based on keywords,");
    println!("analyzes the corpus, and generates custom LLM models for research purposes.");
    println!("Models can be used with Ollama, LMStudio, or similar local LLM platforms.");

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "WORKFLOW".yellow().bold());
    println!(
        "  {}  Search PubMed XML files for articles matching your keywords",
        "1.".cyan()
    );
    println!(
        "  {}  Review statistics and refine keywords if necessary",
        "2.".cyan()
    );
    println!(
        "  {}  Generate Ollama Modelfile and training corpus",
        "3.".cyan()
    );
    println!(
        "  {}  (Optional) Package model files for distribution",
        "4.".cyan()
    );
    println!(
        "  {}  Train or load model with Ollama/LMStudio",
        "5.".cyan()
    );
    println!(
        "  {}  Interact with Python visualizations for insights",
        "6.".cyan()
    );

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "COMMON COMMANDS".yellow().bold());

    println!("  {} Search with keywords (AND logic):", "•".green());
    println!(
        "    {}",
        "curalit search -k \"cancer treatment\" -k \"immunotherapy\" -d ./data -o results".cyan()
    );

    println!("\n  {} Search with OR logic:", "•".green());
    println!(
        "    {}",
        "curalit search -k \"diabetes\" -k \"glucose\" -d ./data -o results --logic OR".cyan()
    );

    println!("\n  {} Search using keywords from file:", "•".green());
    println!(
        "    {}",
        "curalit search -f keywords.txt -d ./data -o results".cyan()
    );

    println!("\n  {} Resume interrupted search:", "•".green());
    println!(
        "    {}",
        "curalit search -k \"cancer\" -d ./data -o results --resume".cyan()
    );

    println!("\n  {} Generate statistics:", "•".green());
    println!("    {}", "curalit stats -c results.csv".cyan());

    println!("\n  {} Generate Ollama Modelfile:", "•".green());
    println!(
        "    {}",
        "curalit generate -c results.csv -m my-medical-llm -b llama3".cyan()
    );

    println!("\n  {} Generate and package model:", "•".green());
    println!(
        "    {}",
        "curalit generate -c results.csv -m my-model -b llama3 --package".cyan()
    );

    println!("\n  {} Package existing model files:", "•".green());
    println!("    {}", "curalit package -m my-medical-llm".cyan());
    println!("    {}", "curalit package -m my-medical-llm -f zip".cyan());

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "KEY FEATURES".yellow().bold());

    println!(
        "  {} {}",
        "•".green(),
        "Memory-efficient streaming XML parser for large datasets"
    );
    println!(
        "  {} {}",
        "•".green(),
        "Searches all fields: titles, abstracts, MeSH terms, chemicals, authors"
    );
    println!(
        "  {} {}",
        "•".green(),
        "Configurable keyword logic (AND/OR)"
    );
    println!(
        "  {} {}",
        "•".green(),
        "Checkpoint system for resumable operations"
    );
    println!(
        "  {} {}",
        "•".green(),
        "Statistical analysis with threshold warnings (>1000 articles)"
    );
    println!(
        "  {} {}",
        "•".green(),
        "Automatic Python visualization generation (Plotly, Seaborn)"
    );
    println!(
        "  {} {}",
        "•".green(),
        "Model packaging for easy distribution (tar.gz or zip)"
    );
    println!("  {} {}", "•".green(), "Progress bars and colorized output");
    println!(
        "  {} {}",
        "•".green(),
        "Fast parallel processing with Rayon"
    );

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "Docker".yellow().bold());
    println!(" CuraLit uses Docker to host its Qdrant database services.");
    println!("  {} {}: {}", "•".green(), "Command".red(), "docker run -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage qdrant/qdrant".cyan());

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "CORPORA".yellow().bold());

    println!(
        " Corpus files are formatted in XML and are downloaded from NCBI from the following URLs:"
    );

    println!(
        "  {} {}: {}",
        "•".green(),
        "Baseline URL".red(),
        "https://ftp.ncbi.nlm.nih.gov/pubmed/baseline/".cyan()
    );

    println!(
        "  {} {}: {}",
        "•".green(),
        "Updatefiles URL".red(),
        "https://ftp.ncbi.nlm.nih.gov/pubmed/updatefiles/".cyan()
    );

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "FILE FORMATS".yellow().bold());

    println!("  {} Input: PubMed XML files (pubmed*.xml)", "•".green());
    println!(
        "  {} Checkpoint: CSV with PMID, title, authors, abstract, keywords",
        "•".green()
    );
    println!(
        "  {} Packages: .tar.gz or .zip archives (distributable)",
        "•".green()
    );
    println!(
        "  {} Output: Ollama Modelfile + training data (JSONL)",
        "•".green()
    );
    println!("  {} Statistics: JSON + log file", "•".green());
    println!("  {} Visualizations: Python scripts (.py)", "•".green());

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "EXAMPLES".yellow().bold());

    println!("  {} Finding cancer immunotherapy research:", "1.".cyan());
    println!(
        "     {}",
        "curalit search -k \"cancer\" -k \"immunotherapy\" -d ./pubmed_data".cyan()
    );

    println!("\n  {} Broad diabetes research (OR logic):", "2.".cyan());
    println!(
        "     {}",
        "curalit search -k \"diabetes\" -k \"insulin\" -k \"glucose\" --logic OR -d ./data".cyan()
    );

    println!("\n  {} Check if keywords are too broad:", "3.".cyan());
    println!("     {}", "curalit stats -c diabetes_results.csv".cyan());
    println!(
        "     {}",
        "# If >1000 articles, refine keywords and search again".cyan()
    );

    println!("\n  {} Create and package specialized LLM:", "4.".cyan());
    println!(
        "     {}",
        "curalit generate -c results.csv -m cardiology-llm -b llama3 --package".cyan()
    );
    println!(
        "     {}",
        "# Creates model files + cardiology-llm_distributable.tar.gz".cyan()
    );

    println!("\n  {} Share model with colleagues:", "5.".cyan());
    println!(
        "     {}",
        "# Send the .tar.gz or .zip file to others".cyan()
    );
    println!(
        "     {}",
        "# Recipients extract and run: ollama create cardiology-llm -f Modelfile_*".cyan()
    );

    println!("\n  {} Use the model locally:", "6.".cyan());
    println!(
        "     {}",
        "ollama create cardiology-llm -f Modelfile_cardiology-llm_*".cyan()
    );
    println!("     {}", "ollama run cardiology-llm".cyan());

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "TIPS FOR NOVICE RESEARCHERS".yellow().bold());

    println!("  {} Start with specific keywords (2-4 terms)", "•".green());
    println!(
        "  {} Use AND logic for focused results, OR for broader exploration",
        "•".green()
    );
    println!(
        "  {} Check statistics before generating models",
        "•".green()
    );
    println!(
        "  {} If >1000 articles: add more specific keywords",
        "•".green()
    );
    println!(
        "  {} If <50 articles: use OR logic or broader terms",
        "•".green()
    );
    println!(
        "  {} Explore Python visualizations to understand your corpus",
        "•".green()
    );
    println!(
        "  {} Test models with simple questions before complex analysis",
        "•".green()
    );

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "TROUBLESHOOTING".yellow().bold());

    println!("  {} Search interrupted: Use --resume flag", "•".green());
    println!(
        "  {} Too many/few results: Adjust keywords and logic",
        "•".green()
    );
    println!(
        "  {} Out of memory: CuraLit streams XML (shouldn't happen)",
        "•".green()
    );
    println!(
        "  {} Model too large: Filter to fewer, more relevant articles",
        "•".green()
    );

    println!("\n{}", "═".repeat(80).cyan());
    println!("\n{}\n", "TESTING".yellow().bold());

    println!(
        "{}",
        "CuraLit includes 71+ comprehensive tests covering all major functionality:".bright_white()
    );
    println!(
        "  {} Unit tests (61): Keyword matching, parsing, modelfile generation, checkpoints",
        "•".green()
    );
    println!(
        "  {} Integration tests (14): End-to-end workflows with real data",
        "•".green()
    );
    println!(
        "  {} RAG tests: Vector database and semantic search",
        "•".green()
    );
    println!("  {} Database tests: SQLite fact verification", "•".green());

    println!("\n  {} Quick test run (recommended):", "1.".cyan());
    println!("     {}", "./run_tests.sh".cyan());
    println!(
        "     {}",
        "# Runs all unit tests (~30 seconds)".bright_black()
    );

    println!(
        "\n  {} Full test suite with integration tests:",
        "2.".cyan()
    );
    println!("     {}", "./run_tests.sh --full".cyan());
    println!(
        "     {}",
        "# Includes end-to-end workflows (~2-3 minutes)".bright_black()
    );

    println!(
        "\n  {} Include RAG tests (requires Qdrant + Ollama):",
        "3.".cyan()
    );
    println!("     {}", "./run_tests.sh --rag".cyan());

    println!("\n  {} Run specific test suite:", "4.".cyan());
    println!("     {}", "cargo test --test article_test".cyan());
    println!(
        "     {}",
        "# Options: article_test, parser_test, modelfile_test, checkpoint_test".bright_black()
    );

    println!("\n  {} Run all Rust unit tests:", "5.".cyan());
    println!("     {}", "cargo test".cyan());

    println!("\n  {} Comprehensive integration tests:", "6.".cyan());
    println!("     {}", "cd tests && ./comprehensive_test.sh".cyan());
    println!(
        "     {}",
        "# Tests 14 complete workflows with colored output".bright_black()
    );

    println!("\n  {} Test with sample data:", "7.".cyan());
    println!(
        "     {}",
        "curalit search -k \"methanol\" -d ./data -o test_results".cyan()
    );

    println!("\n  {} Verify installation:", "BEST PRACTICE:".yellow());
    println!(
        "     {}",
        "Run './run_tests.sh' after building to ensure everything works correctly!".cyan()
    );

    println!(
        "\n  {} For complete testing documentation:",
        "INFO:".yellow()
    );
    println!("     {}", "See tests/README.md".cyan());
    println!("     {}", "See TESTING_SUMMARY.md".cyan());

    println!("\n{}", "═".repeat(80).cyan());
    println!(
        "\n{}\n",
        "Creating a Virtual Environment for Python Code"
            .yellow()
            .bold()
    );

    println!(
        "  {} Create virtual environment: {}",
        "•".green(),
        "python3 -m venv venv".cyan()
    );
    println!(
        "  {} Activate virtual environment: {}",
        "•".green(),
        "source venv/bin/activate (Linux/macOS) or venv\\Scripts\\activate (Windows)".cyan()
    );
    println!(
        "  {} Install dependencies: {}",
        "•".green(),
        "pip install plotly pandas".cyan()
    );
    println!(
        "  {} Deactivate virtual environment (after running visualization code): {}",
        "•".green(),
        "deactivate".cyan()
    );

    println!("\n{}", "═".repeat(80).cyan());
    println!(
        "\nGitHub Repository: {}",
        "https://github.com/developmentAC/curalit"
            .blue()
            .underline()
    );
    println!(
        "{}\n",
        "For more info see README.MD".yellow().bold().underline()
    );
    println!();
}

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// CuraLit: Literature-Driven LLM Generator
///
/// Extract relevant articles from PubMed XML datasets and generate custom LLMs
/// for research purposes using Ollama, LMStudio, or similar platforms.
#[derive(Parser, Debug)]
#[command(name = "curalit")]
#[command(version = "0.4.0")]
#[command(about = "Generate custom LLMs from PubMed literature", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display comprehensive help with examples and workflow
    BigHelp,

    /// Search PubMed XML files for articles matching keywords
    Search {
        /// Keywords to search for (can be specified multiple times)
        #[arg(short = 'k', long = "keyword", value_name = "KEYWORD")]
        keywords: Vec<String>,

        /// File containing keywords (one per line)
        #[arg(short = 'f', long = "keywords-file", value_name = "FILE")]
        keywords_file: Option<PathBuf>,

        /// Directory containing PubMed XML files
        #[arg(
            short = 'd',
            long = "data-dir",
            value_name = "DIR",
            default_value = "./data"
        )]
        data_dir: PathBuf,

        /// Output name (creates <name>.csv, <name>_stats.json, etc.)
        #[arg(
            short = 'o',
            long = "output",
            value_name = "NAME",
            default_value = "results"
        )]
        output: String,

        /// Keyword matching logic (AND: all must match, OR: any can match)
        #[arg(short = 'l', long = "logic", value_enum, default_value = "and")]
        logic: KeywordLogic,

        /// Resume from existing checkpoint file
        #[arg(short = 'r', long = "resume")]
        resume: bool,

        /// Warning threshold for article count
        #[arg(
            short = 't',
            long = "threshold",
            value_name = "NUM",
            default_value = "1000"
        )]
        threshold: usize,
    },

    /// Generate statistics and visualizations from checkpoint file
    Stats {
        /// Checkpoint CSV file to analyze
        #[arg(short = 'c', long = "checkpoint", value_name = "FILE")]
        checkpoint_file: PathBuf,
    },

    /// Generate Ollama Modelfile and training data
    Generate {
        /// Checkpoint CSV file containing articles
        #[arg(short = 'c', long = "checkpoint", value_name = "FILE")]
        checkpoint_file: PathBuf,

        /// Name for the generated model
        #[arg(short = 'm', long = "model-name", value_name = "NAME")]
        model_name: String,

        /// Base model to fine-tune (e.g., llama3, mistral, phi3)
        #[arg(
            short = 'b',
            long = "base-model",
            value_name = "MODEL",
            default_value = "llama3"
        )]
        base_model: String,

        /// Create distributable package (tar.gz or zip)
        #[arg(short = 'p', long = "package")]
        package: bool,

        /// Package format (tar or zip)
        #[arg(
            short = 'f',
            long = "package-format",
            value_name = "FORMAT",
            default_value = "tar",
            value_enum
        )]
        package_format: Option<PackageFormat>,
    },

    /// Package model files for distribution
    Package {
        /// Model name (used to find generated files)
        #[arg(short = 'm', long = "model-name", value_name = "NAME")]
        model_name: String,

        /// Output directory containing model files
        #[arg(
            short = 'd',
            long = "output-dir",
            value_name = "DIR",
            default_value = "0_out"
        )]
        output_dir: PathBuf,

        /// Package format (tar or zip)
        #[arg(
            short = 'f',
            long = "format",
            value_name = "FORMAT",
            default_value = "tar",
            value_enum
        )]
        format: PackageFormat,

        /// Output filename (without extension)
        #[arg(short = 'o', long = "output", value_name = "FILE")]
        output_name: Option<String>,
    },

    /// Build RAG (Retrieval-Augmented Generation) index from checkpoint
    RagBuild {
        /// Checkpoint CSV file containing articles
        #[arg(short = 'c', long = "checkpoint", value_name = "FILE")]
        checkpoint_file: PathBuf,

        /// Ollama embedding model (default: nomic-embed-text)
        #[arg(
            short = 'e',
            long = "embedding-model",
            value_name = "MODEL",
            default_value = "nomic-embed-text"
        )]
        embedding_model: String,

        /// Qdrant storage path
        #[arg(
            short = 's',
            long = "storage",
            value_name = "DIR",
            default_value = "0_out/qdrant_storage"
        )]
        storage_path: PathBuf,

        /// Collection name in Qdrant
        #[arg(
            short = 'n',
            long = "collection-name",
            value_name = "NAME",
            default_value = "curalit_articles"
        )]
        collection_name: String,
    },

    /// Query the RAG index for relevant passages
    RagQuery {
        /// Query text to search for
        #[arg(short = 'q', long = "query", value_name = "TEXT")]
        query: String,

        /// Qdrant storage path
        #[arg(
            short = 's',
            long = "storage",
            value_name = "DIR",
            default_value = "0_out/qdrant_storage"
        )]
        storage_path: PathBuf,

        /// Collection name in Qdrant
        #[arg(
            short = 'n',
            long = "collection-name",
            value_name = "NAME",
            default_value = "curalit_articles"
        )]
        collection_name: String,

        /// Ollama embedding model
        #[arg(
            short = 'e',
            long = "embedding-model",
            value_name = "MODEL",
            default_value = "nomic-embed-text"
        )]
        embedding_model: String,

        /// Number of results to retrieve
        #[arg(short = 'k', long = "top-k", value_name = "NUM", default_value = "5")]
        top_k: usize,
    },

    /// Generate answer using RAG (retrieve relevant passages + LLM generation)
    RagGenerate {
        /// Question to answer
        #[arg(short = 'q', long = "query", value_name = "TEXT")]
        query: String,

        /// Ollama model for generation (e.g., llama3, mistral)
        #[arg(
            short = 'm',
            long = "model",
            value_name = "MODEL",
            default_value = "llama3"
        )]
        model: String,

        /// Qdrant storage path
        #[arg(
            short = 's',
            long = "storage",
            value_name = "DIR",
            default_value = "0_out/qdrant_storage"
        )]
        storage_path: PathBuf,

        /// Collection name in Qdrant
        #[arg(
            short = 'n',
            long = "collection-name",
            value_name = "NAME",
            default_value = "curalit_articles"
        )]
        collection_name: String,

        /// Ollama embedding model
        #[arg(
            short = 'e',
            long = "embedding-model",
            value_name = "MODEL",
            default_value = "nomic-embed-text"
        )]
        embedding_model: String,

        /// Number of passages to retrieve for context
        #[arg(short = 'k', long = "top-k", value_name = "NUM", default_value = "5")]
        top_k: usize,

        /// Use SQLite database for fact verification (PMID, authors, DOI, etc.)
        #[arg(long = "use-db", value_name = "DB_PATH")]
        use_db: Option<PathBuf>,
    },

    /// Package RAG model with vector database for distribution
    RagPackage {
        /// Collection name in Qdrant
        #[arg(
            short = 'n',
            long = "collection-name",
            value_name = "NAME",
            default_value = "curalit_articles"
        )]
        collection_name: String,

        /// Qdrant storage path
        #[arg(
            short = 's',
            long = "storage",
            value_name = "DIR",
            default_value = "qdrant_storage"
        )]
        storage_path: PathBuf,

        /// Output package name (without extension)
        #[arg(short = 'o', long = "output", value_name = "NAME")]
        output_name: Option<String>,

        /// Package format (tar or zip)
        #[arg(
            short = 'f',
            long = "format",
            value_name = "FORMAT",
            default_value = "tar",
            value_enum
        )]
        format: PackageFormat,

        /// Output directory for the package
        #[arg(
            short = 'd',
            long = "output-dir",
            value_name = "DIR",
            default_value = "0_out"
        )]
        output_dir: PathBuf,
    },

    /// Build SQLite database from articles matching keywords for fact verification
    DbBuild {
        /// Keywords to search for (can be specified multiple times)
        #[arg(short = 'k', long = "keyword", value_name = "KEYWORD")]
        keywords: Vec<String>,

        /// File containing keywords (one per line)
        #[arg(short = 'f', long = "keywords-file", value_name = "FILE")]
        keywords_file: Option<PathBuf>,

        /// Directory containing PubMed XML files
        #[arg(
            short = 'd',
            long = "data-dir",
            value_name = "DIR",
            default_value = "./data"
        )]
        data_dir: PathBuf,

        /// Output directory for database
        #[arg(
            short = 'o',
            long = "output-dir",
            value_name = "DIR",
            default_value = "0_out"
        )]
        output_dir: PathBuf,

        /// Database name (without .db extension)
        #[arg(
            short = 'n',
            long = "db-name",
            value_name = "NAME",
            default_value = "curalit"
        )]
        db_name: String,

        /// Keyword matching logic (AND: all must match, OR: any can match)
        #[arg(short = 'l', long = "logic", value_enum, default_value = "and")]
        logic: KeywordLogic,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum PackageFormat {
    /// Create .tar.gz archive
    Tar,
    /// Create .zip archive
    Zip,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum KeywordLogic {
    /// All keywords must match (more specific)
    And,
    /// Any keyword can match (broader results)
    Or,
}

impl std::fmt::Display for KeywordLogic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeywordLogic::And => write!(f, "AND"),
            KeywordLogic::Or => write!(f, "OR"),
        }
    }
}

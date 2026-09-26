# New Database Feature for Fact Verification

## Overview

CuraLit v0.3.1 now includes a SQLite database feature to help verify factual information (PMID, Authors, DOI, etc.) that AI models often confuse during literature reviews. This addresses the common problem where AI models "muddle" reference information when generating literature reviews.

## Key Features

### 1. SQLite Database Creation (`db-build` Command)

Build a SQLite database from PubMed XML files filtered by keywords. The database stores:
- PMID (PubMed ID)
- Title
- Authors
- Abstract
- Journal
- Publication Date
- DOI
- MeSH Terms
- Keywords

The database includes:
- Indexed columns for fast PMID, author, and DOI lookups
- Full-text search capabilities on titles and abstracts
- Automatic timestamped filenames

### 2. RAG Integration with Fact Verification

When using the RAG (Retrieval-Augmented Generation) system, you can now enable database fact-checking with the `--use-db` flag. The system:
- Extracts PMIDs mentioned in AI-generated answers
- Verifies them against the database
- Displays complete, accurate citation information
- Warns if PMIDs are not found in the database

### 3. Improved Output Organization

- All HTML visualization files are now automatically placed in an `html/` subdirectory within the output directory
- Cleaner output directory structure

### 4. Human-Readable Timestamps

Filenames now use a human-readable timestamp format: `DDMmmYYYY_HHMMSS`
- Example: `curalit_27May2026_143045.db`
- Instead of: `20260527_143045`

## Usage Examples

### Building a Database

```bash
# Build database with specific keywords
curalit db-build -k "cancer" -k "immunotherapy" -d ./data -o 0_out -n my_research

# Build database from keywords file
curalit db-build -f keywords.txt -d ./data -o 0_out -n literature_db

# Build database with OR logic (broader results)
curalit db-build -k "diabetes" -k "insulin" -k "glucose" --logic or -d ./data

# Build database with AND logic (broader results)
curalit db-build -k "diabetes" -k "insulin" -k "glucose" --logic and -d ./data
```

**Output**: Creates `0_out/my_research_27May2026_143045.db`

### Using Database with RAG for Fact Verification

```bash
# Generate answer with fact verification
curalit rag-generate \
  -q "What are the key findings about PMID 12345678?" \
  -m llama3 \
  --use-db 0_out/my_research_27May2026_143045.db

# Without database (standard RAG)
curalit rag-generate -q "Your question" -m llama3
```

**When database verification is enabled:**
1. RAG retrieves relevant passages and generates an answer
2. The system extracts any PMIDs mentioned in the answer
3. Each PMID is verified against the database
4. Accurate citation information is displayed
5. Missing or incorrect PMIDs are flagged

### Example Output with Database Verification

```
════════════════════════════════════════════════════════════════════════════════
Answer:

According to the retrieved literature, PMID 12345678 discusses...

════════════════════════════════════════════════════════════════════════════════
Database Verification (PMID/DOI Fact-Checking)
════════════════════════════════════════════════════════════════════════════════

✓ Verified PMID: 12345678

Verified Citations:

────────────────────────────────────────────────────────────────────────────────
PMID: 12345678
Title: Novel immunotherapy approaches in cancer treatment
Authors: Smith, J.; Johnson, A.; Brown, K.
Journal: Nature Medicine
Date: 2024-03-15
DOI: 10.1038/nm.1234
────────────────────────────────────────────────────────────────────────────────
```

## Workflow Example

### 1. Build Database from Research Keywords

```bash
curalit db-build \
  -k "cancer immunotherapy" \
  -k "checkpoint inhibitors" \
  -k "PD-1" \
  -d ./pubmed_data \
  -o 0_out \
  -n cancer_research
```

### 2. Build RAG Index from Checkpoint

```bash
curalit search -k "cancer immunotherapy" -d ./pubmed_data -o cancer_articles
curalit rag-build -c 0_out/cancer_articles_27May2026_143045.csv
```

### 3. Query with Fact Verification

```bash
curalit rag-generate \
  -q "What are the clinical outcomes of PD-1 inhibitors according to PMID 34567890?" \
  -m llama3 \
  --use-db 0_out/cancer_research_27May2026_143045.db
```

## Database Schema

### Articles Table
```sql
CREATE TABLE articles (
    pmid TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    authors TEXT,
    abstract TEXT,
    journal TEXT,
    pub_date TEXT,
    doi TEXT,
    mesh_terms TEXT,
    keywords TEXT
)
```

### Full-Text Search
```sql
CREATE VIRTUAL TABLE articles_fts USING fts5(
    pmid,
    title,
    abstract
)
```

## Benefits

1. **Fact Accuracy**: Verify PMIDs, authors, and DOIs against the source database
2. **Reduced Hallucination**: Catch AI-generated incorrect reference information
3. **Research Integrity**: Ensure literature reviews are based on accurate citations
4. **Efficient Lookup**: Fast indexed queries for verification
5. **Portable**: Single SQLite file contains all article metadata

## Technical Details

### Database Creation
- Streaming XML parser (memory-efficient for large datasets)
- Batch inserts (1000 articles at a time for performance)
- Automatic indexing on PMID, authors, and DOI
- Full-text search indexing for semantic queries
- Progress bar with file-by-file processing status

### Fact Verification
- Regex-based PMID extraction from AI responses
- Pattern: `\b(?:PMID:?\s*)?(\d{7,8})\b`
- Matches formats: "PMID 12345678", "PMID: 12345678", "12345678"
- Real-time database lookups
- Color-coded verification status

### File Organization
- Database files: `0_out/[name]_[timestamp].db`
- HTML visualizations: `0_out/html/[name]_[timestamp]_*.html`
- Timestamp format: `DDMmmYYYY_HHMMSS` (e.g., `27May2026_143045`)

## Limitations

- Database only verifies PMIDs that exist in your filtered corpus
- Does not validate PMIDs against external PubMed database
- Best used with comprehensive keyword searches
- Database size depends on number of filtered articles

## Future Enhancements

Potential features for future versions:
- External PubMed API verification
- DOI-based lookups
- Author name disambiguation
- Citation graph analysis
- Export to BibTeX format

## Troubleshooting

### Database Creation Issues

**Problem**: No XML files found
```bash
# Solution: Check data directory path
ls -la ./data/*.xml
```

**Problem**: No articles matched keywords
```bash
# Solution: Use broader keywords or OR logic
curalit db-build -k "broad_term" --logic OR -d ./data
```

### Verification Issues

**Problem**: PMIDs not found in database
- Cause: Article not in filtered corpus
- Solution: Rebuild database with broader keywords or check if PMID exists in source XML

**Problem**: Database file not found
```bash
# Solution: Check output directory and use absolute path
curalit rag-generate -q "question" --use-db /full/path/to/database.db
```

## Command Reference

### db-build
```
curalit db-build [OPTIONS]

OPTIONS:
  -k, --keyword <KEYWORD>         Keywords to search for (can be used multiple times)
  -f, --keywords-file <FILE>      File containing keywords (one per line)
  -d, --data-dir <DIR>            Directory containing PubMed XML files [default: ./data]
  -o, --output-dir <DIR>          Output directory for database [default: 0_out]
  -n, --db-name <NAME>            Database name without .db extension [default: curalit]
  -l, --logic <LOGIC>             Keyword matching logic: and or or [default: and]
```

### rag-generate (updated)
```
curalit rag-generate [OPTIONS]

NEW OPTION:
  --use-db <DB_PATH>              Enable database fact verification with specified database file
```

## See Also

- [QUICKSTART.md](QUICKSTART.md) - Getting started guide
- [README.md](README.md) - Main documentation
- [rag_workflow.sh](rag_workflow.sh) - RAG workflow examples

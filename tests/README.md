# CuraLit Test Suite Documentation

## Overview

This document describes the comprehensive testing suite for CuraLit, a literature-driven LLM generator that searches PubMed XML datasets and creates custom models for Ollama.

## Test Structure

The test suite includes multiple layers of testing:

```
tests/
├── article_test.rs           # Unit tests for Article struct and keyword matching
├── parser_test.rs            # Unit tests for PubMed XML parser
├── modelfile_test.rs         # Unit tests for Modelfile generation
├── checkpoint_test.rs        # Unit tests for checkpoint/resume functionality
├── database_test.rs          # Integration tests for SQLite database
├── rag_integration_test.rs   # Integration tests for RAG system (requires services)
├── integration_test.sh       # Basic integration tests (deprecated, use comprehensive_test.sh)
└── comprehensive_test.sh     # Complete end-to-end workflow tests
```

## Running Tests

### Quick Start

```bash
# Run all Rust unit tests
cargo test

# Run all Rust tests including ignored (integration) tests
cargo test -- --include-ignored

# Run comprehensive end-to-end tests
./tests/comprehensive_test.sh
```

### Individual Test Suites

#### Unit Tests (Rust)

```bash
# Run specific test file
cargo test --test article_test
cargo test --test parser_test
cargo test --test modelfile_test
cargo test --test checkpoint_test
cargo test --test database_test

# Run specific test
cargo test test_single_keyword_match
cargo test test_parse_complete_article

# Run with output
cargo test -- --nocapture
```

#### Integration Tests (Bash)

```bash
# Run comprehensive test suite
cd tests
./comprehensive_test.sh

# Run original integration tests
./integration_test.sh
```

#### RAG Integration Tests (requires services)

RAG tests require external services to be running:
- Qdrant (vector database): `docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant`
- Ollama with embedding model: `ollama pull nomic-embed-text`

```bash
# Run RAG tests (only when services are available)
cargo test --test rag_integration_test -- --ignored
```

## Test Coverage

### 1. Article and Keyword Matching Tests (`article_test.rs`)

**Purpose**: Verify that keyword matching logic works correctly across all article fields.

**Test Cases**:
- ✅ Single keyword matching (case-insensitive)
- ✅ AND logic (all keywords must match)
- ✅ OR logic (any keyword can match)
- ✅ Searches across all fields: title, abstract, authors, MeSH terms, chemicals
- ✅ CSV serialization/deserialization
- ✅ Training data format generation
- ✅ Edge cases: empty keywords, minimal articles, special characters
- ✅ Partial keyword matching (substring)

**Key Tests**:
```rust
test_single_keyword_match()         // Basic keyword search
test_matches_all_keywords_and_logic() // AND logic verification
test_matches_any_keyword_or_logic()   // OR logic verification
test_csv_round_trip()                // Data integrity through CSV
test_training_format()               // JSONL training data generation
```

### 2. XML Parser Tests (`parser_test.rs`)

**Purpose**: Ensure PubMed XML files are parsed correctly with proper error handling.

**Test Cases**:
- ✅ Complete article parsing with all fields
- ✅ Multiple articles in single file
- ✅ Articles with missing fields (abstract, DOI, etc.)
- ✅ Multiple authors, MeSH terms, chemicals
- ✅ Publication date parsing
- ✅ Special characters and XML entities
- ✅ Empty XML files
- ✅ Streaming large files (memory efficiency)
- ✅ Malformed XML handling
- ✅ Non-existent file error handling

**Key Tests**:
```rust
test_parse_complete_article()       // Full article parsing
test_parse_multiple_articles()      // Multiple articles
test_streaming_many_articles()      // Memory efficiency (100 articles)
test_parse_special_characters()     // XML entity handling
```

### 3. Modelfile Generation Tests (`modelfile_test.rs`)

**Purpose**: Verify correct generation of Ollama Modelfiles and training data.

**Test Cases**:
- ✅ Modelfile creation with proper format
- ✅ Training data in JSONL format
- ✅ System prompt generation
- ✅ Different base models (llama3, mistral, gemma)
- ✅ Empty article list handling
- ✅ Single article handling
- ✅ Large article sets (100+ articles)
- ✅ Special characters in model names
- ✅ File naming conventions
- ✅ Complete article data inclusion

**Key Tests**:
```rust
test_modelfile_generation()         // Basic generation
test_training_data_format()         // JSONL validation
test_generation_with_many_articles() // Scalability (100 articles)
test_different_base_models()        // Base model variations
```

### 4. Checkpoint and Resume Tests (`checkpoint_test.rs`)

**Purpose**: Ensure checkpoint system allows resumable operations.

**Test Cases**:
- ✅ Checkpoint file creation
- ✅ Adding articles to checkpoint
- ✅ Loading articles from checkpoint
- ✅ Resuming from checkpoint
- ✅ Multiple resume operations
- ✅ Article count accuracy
- ✅ CSV data integrity (round-trip)
- ✅ Special characters in data
- ✅ Large checkpoints (1000+ articles)
- ✅ Error handling for missing files

**Key Tests**:
```rust
test_resume_from_checkpoint()       // Resume functionality
test_checkpoint_data_integrity()    // Round-trip data validation
test_multiple_resumes()             // Multiple resume sessions
test_large_checkpoint()             // Scalability (1000 articles)
```

### 5. Database Tests (`database_test.rs`)

**Purpose**: Verify SQLite database creation and querying for fact verification.

**Test Cases**:
- ✅ Database creation
- ✅ Article insertion and retrieval
- ✅ Search by PMID, author, DOI
- ✅ Full-text search
- ✅ Database statistics
- ✅ Duplicate PMID handling
- ✅ Batch insertion performance
- ✅ Transaction handling

**Key Tests**:
```rust
test_insert_and_retrieve_articles() // Basic CRUD operations
test_full_text_search()             // Search functionality
test_batch_insert_performance()     // Bulk operations (1000 articles)
test_duplicate_pmid_handling()      // Data integrity
```

### 6. RAG Integration Tests (`rag_integration_test.rs`)

**Purpose**: Test Retrieval-Augmented Generation system with vector database.

**Test Cases** (requires external services):
- ✅ RAG system initialization
- ✅ Text chunking for embeddings
- ✅ Index building with Qdrant
- ✅ Semantic query retrieval
- ✅ Answer generation with LLM
- ✅ Configuration save/load
- ✅ Multiple query handling
- ✅ Service connectivity checks

**Key Tests**:
```rust
test_rag_build_index()              // Vector DB indexing
test_rag_query_retrieval()          // Semantic search
test_rag_generate_answer()          // LLM answer generation
```

### 7. End-to-End Workflow Tests (`comprehensive_test.sh`)

**Purpose**: Validate complete workflows from search to model creation.

**Test Scenarios**:

1. **Basic Search**: Single keyword search
2. **AND Logic Search**: Multiple keywords with AND logic
3. **OR Logic Search**: Multiple keywords with OR logic
4. **Keywords from File**: Loading keywords from external file
5. **Statistics Generation**: Creating stats and visualizations
6. **Modelfile Generation**: Creating Ollama-compatible models
7. **Resume Functionality**: Interrupted search recovery
8. **Database Build**: SQLite database creation from XML
9. **Model Packaging (tar.gz)**: Distributable archive creation
10. **Model Packaging (zip)**: Alternative archive format
11. **BigHelp Command**: Documentation display
12. **Error Handling**: Invalid inputs and edge cases
13. **CSV Validation**: Output format verification

**Running**:
```bash
cd /path/to/curalit
./tests/comprehensive_test.sh
```

**Output**:
- Colored output showing pass/fail for each test
- Detailed test summary with pass rate
- Test logs saved to temporary directory
- Exit code 0 if all pass, 1 if any fail

## Test Data

### Sample PubMed XML

Tests use synthetic PubMed XML with realistic structure:
- Multiple articles with complete metadata
- Various combinations of keywords
- Different publication types
- Edge cases (missing DOIs, single authors, etc.)

### Expected Results

| Test Scenario | Keywords | Logic | Expected Count |
|--------------|----------|-------|----------------|
| Basic cancer search | "cancer" | N/A | 2 articles |
| Cancer AND immunotherapy | "cancer", "immunotherapy" | AND | 1 article |
| Diabetes OR melanoma | "diabetes", "melanoma" | OR | 2 articles |

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
    
    - name: Run unit tests
      run: cargo test
    
    - name: Run integration tests
      run: ./tests/comprehensive_test.sh
    
    - name: Run RAG tests (with services)
      run: |
        docker run -d -p 6333:6333 qdrant/qdrant
        # Wait for services...
        cargo test --test rag_integration_test -- --ignored
```

## Test Maintenance

### Adding New Tests

1. **Unit Tests**: Add to appropriate `*_test.rs` file
2. **Integration Tests**: Add test case to `comprehensive_test.sh`
3. **Update Documentation**: Document new test in this README

### Test Data Updates

If PubMed XML format changes:
1. Update test XML in `comprehensive_test.sh`
2. Update parser tests in `parser_test.rs`
3. Verify all integration tests still pass

### Performance Benchmarks

Current performance expectations:
- Small dataset (10 articles): < 1 second
- Medium dataset (100 articles): < 5 seconds
- Large dataset (1000 articles): < 30 seconds

## Troubleshooting

### Common Issues

**Issue**: RAG tests fail
- **Solution**: Ensure Qdrant and Ollama are running
- **Check**: `docker ps` and `ollama list`

**Issue**: Comprehensive tests timeout
- **Solution**: Increase timeout in script or check system resources

**Issue**: CSV format tests fail
- **Solution**: Check for unexpected newlines or special characters in test data

**Issue**: Package tests fail
- **Solution**: Ensure tar/zip utilities are installed

### Debug Mode

Run tests with verbose output:
```bash
# Rust tests
cargo test -- --nocapture --test-threads=1

# Bash tests
bash -x ./tests/comprehensive_test.sh
```

## Test Metrics

### Coverage Goals

- Unit tests: > 80% code coverage
- Integration tests: All major workflows
- Edge cases: Common error scenarios

### Current Status

| Component | Tests | Coverage |
|-----------|-------|----------|
| Article | 25+ | ~95% |
| Parser | 15+ | ~90% |
| Modelfile | 12+ | ~85% |
| Checkpoint | 15+ | ~90% |
| Database | 8+ | ~85% |
| RAG | 10+ | ~70% |
| Integration | 14 workflows | All paths |

## Contributing

When adding features:
1. Write tests first (TDD)
2. Ensure all existing tests pass
3. Add integration test for new workflow
4. Update this documentation
5. Run full test suite before committing

## License

Tests are part of the CuraLit project and follow the same license.

## Support

For test-related issues:
1. Check this documentation
2. Review test output logs
3. Run tests in debug mode
4. Open issue with test logs and system info

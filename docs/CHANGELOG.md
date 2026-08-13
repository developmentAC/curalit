# Changelog

All notable changes to CuraLit will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-06-22

### Added
- **Keywords Column Enhancement**: The Keywords column in CSV output now automatically populates with the user's search terms that matched each article
  - New method `get_matched_keywords()` identifies which search keywords are present in an article
  - New method `set_matched_keywords()` populates the keywords field with matched search terms
  - Makes it easy to see exactly which keywords triggered each match
  - Works with both AND and OR logic search strategies
  - Updated in both file-based search (`runner.rs`) and database search (`main.rs`)

### Fixed
- **Network Visualization**: Fixed network graph generation failing due to missing keywords data
  - Updated `prepare_articles_data()` to include the `keywords` field in embedded article data
  - Modified network generation to use the keywords field directly instead of re-searching text
  - Network graphs now properly display keyword-article connections
  - Includes fallback to text search for backwards compatibility

### Changed
- Modified article processing to track matched search keywords before saving to CSV
- Updated Article struct methods to support keyword tracking functionality
- Enhanced visualization script to leverage populated keywords field for network graphs

### Documentation
- Updated README.md with explanation of Keywords column behavior
- Updated Quarto presentation with Keywords column information
- Added comprehensive test suite for keyword tracking functionality (7 new tests)

### Tests
- Added `test_get_matched_keywords()` - validates keyword matching logic
- Added `test_set_matched_keywords()` - verifies keyword population
- Added `test_matched_keywords_and_logic()` - tests AND logic scenarios
- Added `test_matched_keywords_or_logic()` - tests OR logic scenarios
- Added `test_matched_keywords_case_insensitive()` - validates case handling
- Added `test_csv_with_matched_keywords()` - ensures CSV serialization works
- All existing tests continue to pass ✓

### Technical Details
The enhancement involved:
1. Adding `get_matched_keywords()` and `set_matched_keywords()` methods to Article struct ([src/article.rs](src/article.rs))
2. Updating search logic to populate keywords before checkpoint saving ([src/runner.rs](src/runner.rs))
3. Updating database search to populate keywords before insertion ([src/main.rs](src/main.rs))
4. Comprehensive test coverage in [tests/article_test.rs](tests/article_test.rs)

## [0.3.5] - 2026-06-20

### Fixed
- **Critical**: Fixed malformed list comprehension in network visualization Python script
  - The article date filtering logic had incorrect syntax that caused Python SyntaxError
  - Changed from broken list comprehension to explicit loop with proper conditional logic
  - Issue: `if article.get('pub_date', ''): year_str = article['pub_date'].split('-')[0]` was incorrectly nested in list comprehension
  - Fix: Converted to proper for-loop with nested if statements

- **Important**: Fixed network graph not showing results for historical data
  - Network visualization was filtering articles by date (last 3 years by default)
  - Historical PubMed data (e.g., from 1970s-1980s) was being excluded
  - Added fallback: if no recent articles found, automatically show all articles
  - Displays informative message: "No articles from last N years found. Showing all X articles."
  
### Added
- **Tests**: Comprehensive automated test suite for visualization script generation ([tests/visualizer_test.rs](tests/visualizer_test.rs))
  - 11 test cases covering script generation, Python syntax validation, data formatting, and function presence
  - Tests verify correct embedding of statistics, search keywords, and article data
  - Python syntax validation using `py_compile` module
  - Tests for special character escaping and empty dataset handling
  - All tests passing ✓

### Changed
- Updated visualization script template to include complete function implementations
- Improved error handling for network visualization with informative messages

### Documentation
- Added [tests/README.md](tests/README.md) entry for visualizer tests
- Updated [README.md](README.md) with pyvis/networkx requirements (already done in v0.3.4)

### Technical Details
The bug was in [src/visualizer.rs](src/visualizer.rs) line 165-171 (before fix):
```python
# BROKEN CODE (caused SyntaxError)
if not show_all:
    filtered_articles = [
        article for article in ARTICLES_DATA
        if article.get('pub_date', ''):
            year_str = article['pub_date'].split('-')[0]  # Invalid nesting
            if year_str.isdigit() and (current_year - int(year_str)) <= recent_years
    ]
```

Fixed code:
```python
# FIXED CODE
if not show_all:
    filtered_articles = []
    for article in ARTICLES_DATA:
        pub_date = article.get('pub_date', '')
        if pub_date:
            year_str = pub_date.split('-')[0]
            if year_str.isdigit() and (current_year - int(year_str)) <= recent_years:
                filtered_articles.append(article)
    
    # Fallback for historical data
    if len(filtered_articles) == 0:
        print(f"  ℹ No articles from last {recent_years} years found. Showing all {len(ARTICLES_DATA)} articles.")
        filtered_articles = ARTICLES_DATA
```

### Why the Network Wasn't Showing Results
The network function filters articles by publication date to avoid overwhelming visualizations with thousands of nodes. By default, it shows only articles from the last 3 years. However:

1. Test data contains articles from 1976 (50 years old)
2. With `recent_years=3`, all 1976 articles were filtered out
3. Result: `filtered_articles` was empty, function returned `None`
4. No network graph was generated

The fix adds a fallback: if the date filter results in zero articles, the function automatically includes all articles and notifies the user. This handles both:
- Historical datasets (research archives, old publications)
- Very recent datasets where filtering is useful

Users can still explicitly control behavior:
- `show_all=True`: Always show all articles
- `recent_years=N`: Adjust the time window
- `max_articles=N`: Limit total nodes displayed

## [0.3.4] - 2026-06-19

### Added
- Interactive keyword-article network graph visualization
- Network shows connections between search keywords and matched PubMed articles
- Clickable nodes that open article pages on PubMed
- Smart filtering: shows recent articles (last 3 years) by default
- Configurable parameters: max_articles, recent_years, show_all, use_mesh

### Changed
- Integrated network graph into main visualization script (removed separate script)
- Updated Statistics struct to include search keywords and full article data
- Version bumped from 0.3.3 to 0.3.4

## [0.3.3] and earlier
See git history for changes prior to 0.3.4.

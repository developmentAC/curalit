#!/bin/bash
################################################################################
# CuraLit Comprehensive Test Suite
# Tests all major workflows and functionality
################################################################################

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Create test directory
TEST_DIR=$(mktemp -d -t curalit-test-XXXXXXXX)
echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║        CuraLit Comprehensive Test Suite                  ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Test directory: ${TEST_DIR}${NC}"
echo ""

################################################################################
# Helper Functions
################################################################################

test_start() {
    ((TESTS_RUN++))
    echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}Test ${TESTS_RUN}: $1${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

test_pass() {
    ((TESTS_PASSED++))
    echo -e "${GREEN}✓ PASSED${NC}: $1"
}

test_fail() {
    ((TESTS_FAILED++))
    echo -e "${RED}✗ FAILED${NC}: $1"
}

check_file_exists() {
    # Expand glob patterns by not quoting in the ls command
    local files=($1)
    if [ -e "${files[0]}" ]; then
        test_pass "File exists: $(basename ${files[0]})"
    else
        test_fail "File missing: $1"
    fi
    return 0  # Always return 0 to prevent set -e from exiting
}

check_file_not_empty() {
    if [ -s "$1" ]; then
        test_pass "File not empty: $(basename $1)"
    else
        test_fail "File is empty: $(basename $1)"
    fi
    return 0  # Always return 0 to prevent set -e from exiting
}

check_line_count() {
    local file=$1
    local expected=$2
    local actual=$(wc -l < "$file" | tr -d ' ')
    
    if [ "$actual" -eq "$expected" ]; then
        test_pass "Line count correct: $actual (expected $expected)"
    else
        test_fail "Line count incorrect: $actual (expected $expected)"
    fi
    return 0  # Always return 0 to prevent set -e from exiting
}

################################################################################
# Test Data Setup
################################################################################

echo -e "${BLUE}Setting up test data...${NC}"

# Create comprehensive test XML with multiple articles
cat > "$TEST_DIR/test_pubmed.xml" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>10001</PMID>
      <Article>
        <Journal>
          <Title>Nature Medicine</Title>
        </Journal>
        <ArticleTitle>Cancer immunotherapy using checkpoint inhibitors</ArticleTitle>
        <Abstract>
          <AbstractText>This study investigates the efficacy of cancer immunotherapy using checkpoint inhibitors in melanoma patients.</AbstractText>
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
          <Month>01</Month>
          <Day>15</Day>
        </ArticleDate>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName>Cancer</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Immunotherapy</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Checkpoint Inhibitors</DescriptorName>
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
  
  <PubmedArticle>
    <MedlineCitation>
      <PMID>10002</PMID>
      <Article>
        <Journal>
          <Title>Journal of Clinical Oncology</Title>
        </Journal>
        <ArticleTitle>Novel approaches to cancer treatment</ArticleTitle>
        <Abstract>
          <AbstractText>Research on novel cancer treatment strategies including targeted therapy.</AbstractText>
        </Abstract>
        <AuthorList>
          <Author>
            <LastName>Williams</LastName>
            <ForeName>Robert</ForeName>
          </Author>
        </AuthorList>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName>Cancer</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Treatment</DescriptorName>
        </MeshHeading>
      </MeshHeadingList>
    </MedlineCitation>
  </PubmedArticle>
  
  <PubmedArticle>
    <MedlineCitation>
      <PMID>10003</PMID>
      <Article>
        <Journal>
          <Title>Diabetes Care</Title>
        </Journal>
        <ArticleTitle>Diabetes management and glucose control</ArticleTitle>
        <Abstract>
          <AbstractText>Study on diabetes management strategies and glucose control in type 2 diabetes patients.</AbstractText>
        </Abstract>
        <AuthorList>
          <Author>
            <LastName>Brown</LastName>
            <ForeName>Emily</ForeName>
          </Author>
        </AuthorList>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName>Diabetes</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Glucose</DescriptorName>
        </MeshHeading>
      </MeshHeadingList>
    </MedlineCitation>
  </PubmedArticle>
  
  <PubmedArticle>
    <MedlineCitation>
      <PMID>10004</PMID>
      <Article>
        <Journal>
          <Title>Cell</Title>
        </Journal>
        <ArticleTitle>Melanoma immunotherapy breakthrough</ArticleTitle>
        <Abstract>
          <AbstractText>Novel melanoma immunotherapy using checkpoint inhibitors shows promising results.</AbstractText>
        </Abstract>
        <AuthorList>
          <Author>
            <LastName>Davis</LastName>
            <ForeName>Michael</ForeName>
          </Author>
        </AuthorList>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName>Melanoma</DescriptorName>
        </MeshHeading>
        <MeshHeading>
          <DescriptorName>Immunotherapy</DescriptorName>
        </MeshHeading>
      </MeshHeadingList>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>
EOF

echo -e "${GREEN}✓ Test data created${NC}"

################################################################################
# Test 1: Basic Search with Single Keyword
################################################################################

test_start "Basic Search with Single Keyword"

cargo run --quiet -- search \
    -k "cancer" \
    -d "$TEST_DIR" \
    -o "test1" \
    > "$TEST_DIR/test1.log" 2>&1

check_file_exists "0_out/test1_*.csv"
if [ $? -eq 0 ]; then
    RESULT_FILE=$(ls 0_out/test1_*.csv 2>/dev/null | head -1)
    check_file_not_empty "$RESULT_FILE"
    # Should find 2 articles with "cancer" (PMIDs 10001, 10002)
    check_line_count "$RESULT_FILE" 3  # Header + 2 articles
fi

################################################################################
# Test 2: Search with AND Logic (Multiple Keywords)
################################################################################

test_start "Search with AND Logic"

cargo run --quiet -- search \
    -k "cancer" \
    -k "immunotherapy" \
    --logic and \
    -d "$TEST_DIR" \
    -o "test2" \
    > "$TEST_DIR/test2.log" 2>&1

check_file_exists "0_out/test2_*.csv"
if [ $? -eq 0 ]; then
    RESULT_FILE=$(ls 0_out/test2_*.csv 2>/dev/null | head -1)
    check_file_not_empty "$RESULT_FILE"
    # Should find 1 article with both "cancer" AND "immunotherapy" (PMID 10001)
    check_line_count "$RESULT_FILE" 2  # Header + 1 article
fi

################################################################################
# Test 3: Search with OR Logic
################################################################################

test_start "Search with OR Logic"

cargo run --quiet -- search \
    -k "diabetes" \
    -k "melanoma" \
    --logic or \
    -d "$TEST_DIR" \
    -o "test3" \
    > "$TEST_DIR/test3.log" 2>&1

check_file_exists "0_out/test3_*.csv"
if [ $? -eq 0 ]; then
    RESULT_FILE=$(ls 0_out/test3_*.csv 2>/dev/null | head -1)
    check_file_not_empty "$RESULT_FILE"
    # Should find 2 articles (PMIDs 10003, 10004)
    check_line_count "$RESULT_FILE" 3  # Header + 2 articles
fi

################################################################################
# Test 4: Keywords from File
################################################################################

test_start "Search with Keywords from File"

# Create keywords file
cat > "$TEST_DIR/keywords.txt" << EOF
immunotherapy
checkpoint
EOF

cargo run --quiet -- search \
    -f "$TEST_DIR/keywords.txt" \
    --logic and \
    -d "$TEST_DIR" \
    -o "test4" \
    > "$TEST_DIR/test4.log" 2>&1

check_file_exists "0_out/test4_*.csv"
if [ $? -eq 0 ]; then
    RESULT_FILE=$(ls 0_out/test4_*.csv 2>/dev/null | head -1)
    check_file_not_empty "$RESULT_FILE"
fi

################################################################################
# Test 5: Generate Statistics
################################################################################

test_start "Generate Statistics"

# Use results from Test 1
CHECKPOINT=$(ls 0_out/test1_*.csv 2>/dev/null | head -1)

if [ -n "$CHECKPOINT" ]; then
    cargo run --quiet -- stats -c "$CHECKPOINT" > "$TEST_DIR/test5.log" 2>&1
    
    # Check for stats files
    STATS_JSON="${CHECKPOINT%.csv}_stats.json"
    STATS_LOG="${CHECKPOINT%.csv}_stats.log"
    VIZ_SCRIPT="${CHECKPOINT%.csv}_visualize.py"
    
    check_file_exists "$STATS_JSON"
    check_file_exists "$STATS_LOG"
    check_file_exists "$VIZ_SCRIPT"
    
    # Verify JSON is valid
    if [ -f "$STATS_JSON" ]; then
        if python3 -m json.tool "$STATS_JSON" > /dev/null 2>&1; then
            test_pass "Statistics JSON is valid"
        else
            test_fail "Statistics JSON is invalid"
        fi
    fi
else
    test_fail "No checkpoint file found for statistics test"
fi

################################################################################
# Test 6: Generate Modelfile
################################################################################

test_start "Generate Modelfile"

CHECKPOINT=$(ls 0_out/test1_*.csv 2>/dev/null | head -1)

if [ -n "$CHECKPOINT" ]; then
    cargo run --quiet -- generate \
        -c "$CHECKPOINT" \
        -m "test-cancer-model" \
        -b "llama3" \
        > "$TEST_DIR/test6.log" 2>&1
    
    # Check for generated files
    check_file_exists "0_out/Modelfile_test-cancer-model_*"
    
    # Find training file
    TRAINING=$(ls 0_out/*_training.jsonl 2>/dev/null | tail -1)
    if [ -n "$TRAINING" ]; then
        check_file_exists "$TRAINING"
        check_file_not_empty "$TRAINING"
        
        # Verify JSONL format
        if head -1 "$TRAINING" | python3 -m json.tool > /dev/null 2>&1; then
            test_pass "Training data is valid JSONL"
        else
            test_fail "Training data is invalid JSONL"
        fi
    fi
    
    # Check system prompt
    PROMPT=$(ls 0_out/*_system_prompt.txt 2>/dev/null | tail -1)
    if [ -n "$PROMPT" ]; then
        check_file_exists "$PROMPT"
        check_file_not_empty "$PROMPT"
    fi
else
    test_fail "No checkpoint file found for generate test"
fi

################################################################################
# Test 7: Resume Functionality
################################################################################

test_start "Resume Functionality"

# Start a search
cargo run --quiet -- search \
    -k "cancer" \
    -d "$TEST_DIR" \
    -o "test7_resume" \
    > "$TEST_DIR/test7a.log" 2>&1

CHECKPOINT=$(ls 0_out/test7_resume_*.csv 2>/dev/null | head -1)

if [ -n "$CHECKPOINT" ]; then
    INITIAL_LINES=$(wc -l < "$CHECKPOINT")
    
    # Manually add a line to simulate partial completion
    # (In real scenario, this would be interrupted mid-search)
    
    # Resume the search
    cargo run --quiet -- search \
        -k "cancer" \
        -d "$TEST_DIR" \
        -o "test7_resume" \
        --resume \
        > "$TEST_DIR/test7b.log" 2>&1
    
    if [ $? -eq 0 ]; then
        test_pass "Resume completed without errors"
    else
        test_fail "Resume encountered errors"
    fi
else
    test_fail "Initial search did not create checkpoint"
fi

################################################################################
# Test 8: Database Build
################################################################################

test_start "Database Build"

cargo run --quiet -- db-build \
    -k "cancer" \
    -k "immunotherapy" \
    --logic or \
    -d "$TEST_DIR" \
    -o "0_out" \
    -n "test_database" \
    > "$TEST_DIR/test8.log" 2>&1

# Check for database file
DB_FILE=$(ls 0_out/test_database_*.db 2>/dev/null | head -1)

if [ -n "$DB_FILE" ]; then
    check_file_exists "$DB_FILE"
    check_file_not_empty "$DB_FILE"
    
    # Verify it's a SQLite database
    if file "$DB_FILE" | grep -q "SQLite"; then
        test_pass "Database is valid SQLite format"
    else
        test_fail "Database is not valid SQLite format"
    fi
else
    test_fail "Database file not created"
fi

################################################################################
# Test 9: Package Model (tar.gz)
################################################################################

test_start "Package Model (tar.gz)"

if [ -n "$CHECKPOINT" ]; then
    cargo run --quiet -- package \
        -m "test-cancer-model" \
        --format tar \
        > "$TEST_DIR/test9.log" 2>&1
    
    PACKAGE=$(ls 0_out/test-cancer-model*.tar.gz 2>/dev/null | head -1)
    
    if [ -n "$PACKAGE" ]; then
        check_file_exists "$PACKAGE"
        check_file_not_empty "$PACKAGE"
        
        # Verify it's a valid tar.gz
        if tar -tzf "$PACKAGE" > /dev/null 2>&1; then
            test_pass "Package is valid tar.gz format"
            
            # Check package contents
            if tar -tzf "$PACKAGE" | grep -q "Modelfile"; then
                test_pass "Package contains Modelfile"
            fi
            if tar -tzf "$PACKAGE" | grep -q "training.jsonl"; then
                test_pass "Package contains training data"
            fi
        else
            test_fail "Package is not valid tar.gz format"
        fi
    else
        test_fail "Package file not created"
    fi
else
    test_fail "No checkpoint for packaging test"
fi

################################################################################
# Test 10: Package Model (zip)
################################################################################

test_start "Package Model (zip)"

if [ -n "$CHECKPOINT" ]; then
    cargo run --quiet -- package \
        -m "test-cancer-model-zip" \
        --format zip \
        > "$TEST_DIR/test10.log" 2>&1
    
    PACKAGE=$(ls 0_out/test-cancer-model-zip*.zip 2>/dev/null | head -1)
    
    if [ -n "$PACKAGE" ]; then
        check_file_exists "$PACKAGE"
        check_file_not_empty "$PACKAGE"
        
        # Verify it's a valid zip
        if unzip -t "$PACKAGE" > /dev/null 2>&1; then
            test_pass "Package is valid zip format"
        else
            test_fail "Package is not valid zip format"
        fi
    else
        test_fail "ZIP package file not created"
    fi
else
    test_fail "No checkpoint for ZIP packaging test"
fi

################################################################################
# Test 11: BigHelp Command
################################################################################

test_start "BigHelp Command"

cargo run --quiet -- big-help > "$TEST_DIR/bighelp.txt" 2>&1

if [ $? -eq 0 ]; then
    test_pass "BigHelp executed successfully"
    
    if [ -s "$TEST_DIR/bighelp.txt" ]; then
        test_pass "BigHelp produced output"
        
        # Check for expected sections
        if grep -q "OVERVIEW" "$TEST_DIR/bighelp.txt"; then
            test_pass "BigHelp contains OVERVIEW section"
        fi
        if grep -q "WORKFLOW" "$TEST_DIR/bighelp.txt"; then
            test_pass "BigHelp contains WORKFLOW section"
        fi
    else
        test_fail "BigHelp produced no output"
    fi
else
    test_fail "BigHelp command failed"
fi

################################################################################
# Test 12: Error Handling - Missing Keywords
################################################################################

test_start "Error Handling - Missing Keywords"

if cargo run --quiet -- search -d "$TEST_DIR" -o "test12" 2>&1 | grep -q "No keywords"; then
    test_pass "Properly handles missing keywords"
else
    test_fail "Did not properly handle missing keywords"
fi

################################################################################
# Test 13: Error Handling - Invalid Directory
################################################################################

test_start "Error Handling - Invalid Directory"

if cargo run --quiet -- search -k "test" -d "/nonexistent/directory" -o "test13" 2>&1 | grep -q -i "error\|failed\|not found"; then
    test_pass "Properly handles invalid directory"
else
    test_fail "Did not properly handle invalid directory"
fi

################################################################################
# Test 14: CSV Format Validation
################################################################################

test_start "CSV Format Validation"

CHECKPOINT=$(ls 0_out/test1_*.csv 2>/dev/null | head -1)

if [ -n "$CHECKPOINT" ]; then
    # Check header
    HEADER=$(head -1 "$CHECKPOINT")
    if echo "$HEADER" | grep -q "PMID.*Title.*Authors"; then
        test_pass "CSV has correct header format"
    else
        test_fail "CSV header format incorrect"
    fi
    
    # Check data rows (should have same number of columns as header)
    HEADER_COLS=$(head -1 "$CHECKPOINT" | awk -F',' '{print NF}')
    DATA_COLS=$(tail -1 "$CHECKPOINT" | awk -F',' '{print NF}')
    
    if [ "$HEADER_COLS" -eq "$DATA_COLS" ]; then
        test_pass "CSV columns consistent"
    else
        test_fail "CSV columns inconsistent (header: $HEADER_COLS, data: $DATA_COLS)"
    fi
else
    test_fail "No CSV file found for validation"
fi

################################################################################
# Test Summary
################################################################################

echo -e "\n${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}                    TEST SUMMARY                           ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}\n"

echo -e "Total Tests Run:    ${BLUE}${TESTS_RUN}${NC}"
echo -e "Tests Passed:       ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Tests Failed:       ${RED}${TESTS_FAILED}${NC}"

PASS_RATE=$((TESTS_PASSED * 100 / TESTS_RUN))
echo -e "Pass Rate:          ${CYAN}${PASS_RATE}%${NC}"

echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                  ALL TESTS PASSED! ✓                      ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
    EXIT_CODE=0
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║              SOME TESTS FAILED! ✗                         ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    EXIT_CODE=1
fi

echo ""
echo -e "${BLUE}Test logs saved in: ${TEST_DIR}${NC}"
echo -e "${YELLOW}Cleanup test directory with: rm -rf ${TEST_DIR}${NC}"
echo ""

exit $EXIT_CODE

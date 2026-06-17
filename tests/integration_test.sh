#!/bin/bash
# Integration tests for CuraLit

set -e

echo "🧪 Running CuraLit Integration Tests"
echo "======================================"

# Create test directory
TEST_DIR=$(mktemp -d)
echo "Test directory: $TEST_DIR"

# Create test XML file
cat > "$TEST_DIR/test_pubmed.xml" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>1</PMID>
      <Article>
        <ArticleTitle>Cancer immunotherapy breakthrough</ArticleTitle>
        <Abstract>
          <AbstractText>This study investigates novel cancer immunotherapy approaches.</AbstractText>
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
        <MeshHeading>
          <DescriptorName>Immunotherapy</DescriptorName>
        </MeshHeading>
      </MeshHeadingList>
    </MedlineCitation>
  </PubmedArticle>
  <PubmedArticle>
    <MedlineCitation>
      <PMID>2</PMID>
      <Article>
        <ArticleTitle>Diabetes management study</ArticleTitle>
        <Abstract>
          <AbstractText>Research on diabetes treatment protocols.</AbstractText>
        </Abstract>
        <AuthorList>
          <Author>
            <LastName>Doe</LastName>
            <ForeName>Jane</ForeName>
          </Author>
        </AuthorList>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName>Diabetes</DescriptorName>
        </MeshHeading>
      </MeshHeadingList>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>
EOF

# Test 1: Search with keywords
echo ""
echo "Test 1: Search with single keyword..."
cargo run -- search -k "cancer" -d "$TEST_DIR" -o "$TEST_DIR/results1"
if [ -f "$TEST_DIR/results1.csv" ]; then
    echo "✓ Test 1 passed: CSV created"
    LINES=$(wc -l < "$TEST_DIR/results1.csv")
    if [ "$LINES" -eq 2 ]; then  # Header + 1 article
        echo "✓ Test 1 passed: Correct article count"
    else
        echo "✗ Test 1 failed: Expected 2 lines, got $LINES"
        exit 1
    fi
else
    echo "✗ Test 1 failed: CSV not created"
    exit 1
fi

# Test 2: Search with AND logic
echo ""
echo "Test 2: Search with AND logic..."
cargo run -- search -k "cancer" -k "immunotherapy" -d "$TEST_DIR" -o "$TEST_DIR/results2" --logic and
if [ -f "$TEST_DIR/results2.csv" ]; then
    echo "✓ Test 2 passed: CSV created"
else
    echo "✗ Test 2 failed: CSV not created"
    exit 1
fi

# Test 3: Search with OR logic
echo ""
echo "Test 3: Search with OR logic..."
cargo run -- search -k "cancer" -k "diabetes" -d "$TEST_DIR" -o "$TEST_DIR/results3" --logic or
if [ -f "$TEST_DIR/results3.csv" ]; then
    echo "✓ Test 3 passed: CSV created"
    LINES=$(wc -l < "$TEST_DIR/results3.csv")
    if [ "$LINES" -eq 3 ]; then  # Header + 2 articles
        echo "✓ Test 3 passed: Correct article count"
    else
        echo "✗ Test 3 failed: Expected 3 lines, got $LINES"
        exit 1
    fi
else
    echo "✗ Test 3 failed: CSV not created"
    exit 1
fi

# Test 4: Generate statistics
echo ""
echo "Test 4: Generate statistics..."
cargo run -- stats -c "$TEST_DIR/results3.csv"
if [ -f "$TEST_DIR/results3_stats.json" ]; then
    echo "✓ Test 4 passed: Stats JSON created"
else
    echo "✗ Test 4 failed: Stats JSON not created"
    exit 1
fi

if [ -f "$TEST_DIR/results3_stats.log" ]; then
    echo "✓ Test 4 passed: Stats log created"
else
    echo "✗ Test 4 failed: Stats log not created"
    exit 1
fi

if [ -f "$TEST_DIR/results3_visualize.py" ]; then
    echo "✓ Test 4 passed: Visualization script created"
else
    echo "✗ Test 4 failed: Visualization script not created"
    exit 1
fi

# Test 5: Generate Modelfile
echo ""
echo "Test 5: Generate Modelfile..."
cd "$TEST_DIR"
cargo run -- generate -c "results3.csv" -m "test-model" -b "llama3"
cd -
if [ -f "$TEST_DIR/Modelfile" ]; then
    echo "✓ Test 5 passed: Modelfile created"
else
    echo "✗ Test 5 failed: Modelfile not created"
    exit 1
fi

if [ -f "$TEST_DIR/results3_training.jsonl" ]; then
    echo "✓ Test 5 passed: Training data created"
else
    echo "✗ Test 5 failed: Training data not created"
    exit 1
fi

# Test 6: BigHelp command
echo ""
echo "Test 6: BigHelp command..."
cargo run -- bighelp > /dev/null
echo "✓ Test 6 passed: BigHelp executed"

# Cleanup
echo ""
echo "Cleaning up test directory..."
rm -rf "$TEST_DIR"

echo ""
echo "======================================"
echo "✓ All integration tests passed!"
echo "======================================"

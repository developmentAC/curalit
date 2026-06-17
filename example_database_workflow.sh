#!/bin/bash
# Example workflow demonstrating the new database feature in CuraLit v0.3.1

set -e

echo "════════════════════════════════════════════════════════════════════════════════"
echo "  CuraLit v0.3.1 - Database Feature Demonstration"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Build database from keywords
echo -e "${CYAN}Step 1: Building SQLite database from PubMed XML files${NC}"
echo "─────────────────────────────────────────────────────────────────────────────"
echo ""
echo "Command:"
echo "  curalit db-build -k \"cancer\" -k \"immunotherapy\" -d ./data -o 0_out -n cancer_research"
echo ""
echo -e "${YELLOW}This will:${NC}"
echo "  • Parse all XML files in ./data directory"
echo "  • Filter articles matching keywords (AND logic)"
echo "  • Create SQLite database: 0_out/cancer_research_27May2026_143045.db"
echo "  • Index PMID, authors, and DOI for fast lookups"
echo "  • Enable full-text search on titles and abstracts"
echo ""

# Uncomment to run:
# ./target/release/curalit db-build -k "cancer" -k "immunotherapy" -d ./data -o 0_out -n cancer_research

echo ""
echo -e "${GREEN}✓ Database created${NC}"
echo ""

# Step 2: Regular article search (for comparison)
echo -e "${CYAN}Step 2: Performing regular article search (for RAG)${NC}"
echo "─────────────────────────────────────────────────────────────────────────────"
echo ""
echo "Command:"
echo "  curalit search -k \"cancer\" -k \"immunotherapy\" -d ./data -o cancer_articles"
echo ""
echo -e "${YELLOW}This creates:${NC}"
echo "  • 0_out/cancer_articles_27May2026_143045.csv"
echo ""

# Uncomment to run:
# ./target/release/curalit search -k "cancer" -k "immunotherapy" -d ./data -o cancer_articles

echo ""
echo -e "${GREEN}✓ Articles extracted${NC}"
echo ""

# Step 3: Build RAG index
echo -e "${CYAN}Step 3: Building RAG index from articles${NC}"
echo "─────────────────────────────────────────────────────────────────────────────"
echo ""
echo "Command:"
echo "  curalit rag-build -c 0_out/cancer_articles_27May2026_143045.csv"
echo ""
echo -e "${YELLOW}This creates:${NC}"
echo "  • 0_out/qdrant_storage/ (vector database)"
echo ""

# Uncomment to run:
# ./target/release/curalit rag-build -c 0_out/cancer_articles_27May2026_143045.csv

echo ""
echo -e "${GREEN}✓ RAG index built${NC}"
echo ""

# Step 4: Generate statistics with HTML visualizations
echo -e "${CYAN}Step 4: Generating statistics and visualizations${NC}"
echo "─────────────────────────────────────────────────────────────────────────────"
echo ""
echo "Command:"
echo "  curalit stats -c 0_out/cancer_articles_27May2026_143045.csv"
echo ""
echo -e "${YELLOW}This creates (NEW):${NC}"
echo "  • 0_out/html/cancer_articles_27May2026_143045_dashboard.html"
echo "  • 0_out/html/cancer_articles_27May2026_143045_year_distribution.html"
echo "  • 0_out/html/cancer_articles_27May2026_143045_mesh_terms.html"
echo "  • 0_out/html/cancer_articles_27May2026_143045_authors.html"
echo "  • 0_out/html/cancer_articles_27May2026_143045_journals.html"
echo "  • 0_out/html/cancer_articles_27May2026_143045_summary.html"
echo ""
echo -e "${YELLOW}Note: All HTML files now in 'html/' subdirectory!${NC}"
echo ""

# Uncomment to run:
# ./target/release/curalit stats -c 0_out/cancer_articles_27May2026_143045.csv

echo ""
echo -e "${GREEN}✓ Visualizations generated${NC}"
echo ""

# Step 5: Query RAG WITHOUT database verification
echo -e "${CYAN}Step 5: RAG query WITHOUT database verification (standard)${NC}"
echo "─────────────────────────────────────────────────────────────────────────────"
echo ""
echo "Command:"
echo "  curalit rag-generate -q \"What are the mechanisms?\" -m llama3"
echo ""
echo -e "${YELLOW}Behavior:${NC}"
echo "  • RAG retrieves relevant passages"
echo "  • LLM generates answer"
echo "  • No fact verification"
echo "  • May include incorrect PMIDs/authors"
echo ""

# Uncomment to run:
# ./target/release/curalit rag-generate -q "What are the mechanisms of cancer immunotherapy?" -m llama3

echo ""

# Step 6: Query RAG WITH database verification (NEW FEATURE)
echo -e "${CYAN}Step 6: RAG query WITH database verification (NEW!)${NC}"
echo "─────────────────────────────────────────────────────────────────────────────"
echo ""
echo "Command:"
echo "  curalit rag-generate \\"
echo "    -q \"What are the mechanisms discussed in PMID 12345678?\" \\"
echo "    -m llama3 \\"
echo "    --use-db 0_out/cancer_research_27May2026_143045.db"
echo ""
echo -e "${YELLOW}Behavior (NEW):${NC}"
echo "  • RAG retrieves relevant passages"
echo "  • LLM generates answer"
echo "  • Extracts PMIDs from answer"
echo "  • Verifies each PMID against database"
echo "  • Displays correct citation information"
echo "  • Warns about incorrect/missing PMIDs"
echo ""
echo -e "${GREEN}✓ Prevents hallucinated references!${NC}"
echo ""

# Uncomment to run:
# ./target/release/curalit rag-generate \
#   -q "What are the mechanisms discussed in PMID 12345678?" \
#   -m llama3 \
#   --use-db 0_out/cancer_research_27May2026_143045.db

echo ""
echo "════════════════════════════════════════════════════════════════════════════════"
echo -e "${GREEN}  Example Output with Database Verification:${NC}"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""
cat << 'EOF'
Answer:

The article PMID 12345678 discusses checkpoint inhibitor mechanisms targeting
PD-1 and CTLA-4 pathways in T-cell activation...

════════════════════════════════════════════════════════════════════════════════
Database Verification (PMID/DOI Fact-Checking)
════════════════════════════════════════════════════════════════════════════════

✓ Verified PMID: 12345678

Verified Citations:

────────────────────────────────────────────────────────────────────────────────
PMID: 12345678
Title: Checkpoint inhibitors in cancer immunotherapy
Authors: Smith, J.; Johnson, A.; Brown, K.
Journal: Nature Medicine
Date: 2024-03-15
DOI: 10.1038/nm.1234
────────────────────────────────────────────────────────────────────────────────
EOF
echo ""
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""

# Summary of new features
echo -e "${CYAN}Summary of New Features in CuraLit v0.3.1:${NC}"
echo "─────────────────────────────────────────────────────────────────────────────"
echo ""
echo -e "${GREEN}1. Database Creation (db-build command):${NC}"
echo "   • Build SQLite databases from PubMed XML files"
echo "   • Filter by keywords with AND/OR logic"
echo "   • Store complete article metadata"
echo "   • Enable fast PMID/author/DOI lookups"
echo "   • Support full-text search"
echo ""
echo -e "${GREEN}2. RAG Fact Verification (--use-db flag):${NC}"
echo "   • Verify PMIDs mentioned in AI answers"
echo "   • Display accurate citation information"
echo "   • Catch hallucinated references"
echo "   • Improve literature review accuracy"
echo ""
echo -e "${GREEN}3. Improved Output Organization:${NC}"
echo "   • All HTML visualizations in html/ subdirectory"
echo "   • Cleaner output directory structure"
echo ""
echo -e "${GREEN}4. Human-Readable Timestamps:${NC}"
echo "   • Format: DDMmmYYYY_HHMMSS"
echo "   • Example: cancer_27May2026_143045.db"
echo "   • Easier file identification"
echo ""
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""
echo "For more information, see:"
echo "  • DATABASE_FEATURE.md - Complete feature documentation"
echo "  • IMPLEMENTATION_SUMMARY.md - Technical details"
echo "  • README.md - General usage"
echo ""
echo "════════════════════════════════════════════════════════════════════════════════"

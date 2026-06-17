#!/bin/bash
################################################################################
# CuraLit - Quick Test Runner
# Runs all test suites with colored output
################################################################################

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           CuraLit Test Suite Runner                       ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

FAILED=0

################################################################################
# Rust Unit Tests
################################################################################

echo -e "${YELLOW}Running Rust Unit Tests...${NC}\n"

if cargo test --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ All Rust unit tests passed${NC}\n"
else
    echo -e "${RED}✗ Some Rust unit tests failed${NC}\n"
    FAILED=1
fi

################################################################################
# Individual Test Suites
################################################################################

echo -e "${YELLOW}Running Individual Test Suites...${NC}\n"

# Article tests
echo -e "${BLUE}Running article tests...${NC}"
if cargo test --test article_test --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ Article tests passed${NC}"
else
    echo -e "${RED}✗ Article tests failed${NC}"
    FAILED=1
fi

# Parser tests
echo -e "${BLUE}Running parser tests...${NC}"
if cargo test --test parser_test --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ Parser tests passed${NC}"
else
    echo -e "${RED}✗ Parser tests failed${NC}"
    FAILED=1
fi

# Modelfile tests
echo -e "${BLUE}Running modelfile tests...${NC}"
if cargo test --test modelfile_test --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ Modelfile tests passed${NC}"
else
    echo -e "${RED}✗ Modelfile tests failed${NC}"
    FAILED=1
fi

# Checkpoint tests
echo -e "${BLUE}Running checkpoint tests...${NC}"
if cargo test --test checkpoint_test --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ Checkpoint tests passed${NC}"
else
    echo -e "${RED}✗ Checkpoint tests failed${NC}"
    FAILED=1
fi

# Database tests
echo -e "${BLUE}Running database tests...${NC}"
if cargo test --test database_test --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ Database tests passed${NC}"
else
    echo -e "${RED}✗ Database tests failed${NC}"
    FAILED=1
fi

echo ""

################################################################################
# Optional: Comprehensive Integration Tests
################################################################################

if [ "$1" = "--full" ] || [ "$1" = "-f" ]; then
    echo -e "${YELLOW}Running Comprehensive Integration Tests...${NC}\n"
    
    if [ -f "./tests/comprehensive_test.sh" ]; then
        ./tests/comprehensive_test.sh
        if [ $? -ne 0 ]; then
            FAILED=1
        fi
    else
        echo -e "${RED}✗ Comprehensive test script not found${NC}"
        FAILED=1
    fi
fi

################################################################################
# Optional: RAG Integration Tests (requires services)
################################################################################

if [ "$1" = "--rag" ] || [ "$2" = "--rag" ]; then
    echo -e "${YELLOW}Running RAG Integration Tests (requires services)...${NC}\n"
    echo -e "${BLUE}Note: Requires Qdrant and Ollama running${NC}"
    
    if cargo test --test rag_integration_test -- --ignored --quiet 2>&1 | grep -q "test result: ok"; then
        echo -e "${GREEN}✓ RAG integration tests passed${NC}"
    else
        echo -e "${YELLOW}⚠ RAG tests skipped or failed (services may not be available)${NC}"
        echo -e "${BLUE}To run RAG tests, ensure:${NC}"
        echo -e "${BLUE}  1. Qdrant is running: docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant${NC}"
        echo -e "${BLUE}  2. Ollama is running with: ollama pull nomic-embed-text${NC}"
    fi
    
    echo ""
fi

################################################################################
# Summary
################################################################################

echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED!${NC}"
    echo ""
    echo -e "${BLUE}Test suite completed successfully.${NC}"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo ""
    echo -e "${YELLOW}Please review the output above for details.${NC}"
    exit 1
fi

echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo -e "  ${CYAN}./run_tests.sh${NC}              - Run unit tests only"
echo -e "  ${CYAN}./run_tests.sh --full${NC}       - Run unit + integration tests"
echo -e "  ${CYAN}./run_tests.sh --rag${NC}        - Include RAG tests (requires services)"
echo -e "  ${CYAN}./run_tests.sh --full --rag${NC} - Run all tests"
echo ""

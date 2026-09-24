#!/bin/bash
# RAG Workflow Script for CuraLit
# This script automates the setup and execution of RAG (Retrieval-Augmented Generation) 
# after running the search command.
#
# Usage: ./rag_workflow.sh <checkpoint_file> [collection_name] [model_name]
#
# Example: ./rag_workflow.sh results_20260526_151054.csv

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored messages
print_info() {
    echo -e "${CYAN}• $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_header() {
    echo -e "\n${CYAN}═══════════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════════${NC}\n"
}

# Check if checkpoint file is provided
if [ $# -lt 1 ]; then
    print_error "Usage: $0 <checkpoint_file> [collection_name] [model_name]"
    echo "Example: $0 results_20260526_151054.csv"
    exit 1
fi

CHECKPOINT_FILE="$1"
COLLECTION_NAME="${2:-curalit_articles}"
LLM_MODEL="${3:-llama3}"

# Check if checkpoint file exists
if [ ! -f "$CHECKPOINT_FILE" ]; then
    print_error "Checkpoint file not found: $CHECKPOINT_FILE"
    echo "Please run the search command first:"
    echo "  curalit search -k \"your keywords\" -d ./data -o results"
    exit 1
fi

print_header "CuraLit RAG Workflow Automation"

# Find curalit binary
CURALIT_BIN=""
if command -v curalit &> /dev/null; then
    CURALIT_BIN="curalit"
    print_success "Found curalit in PATH"
elif [ -f "./target/release/curalit" ]; then
    CURALIT_BIN="./target/release/curalit"
    print_success "Found curalit at ./target/release/curalit"
elif [ -f "./curalit" ]; then
    CURALIT_BIN="./curalit"
    print_success "Found curalit at ./curalit"
else
    print_error "curalit binary not found!"
    echo "Please ensure curalit is either:"
    echo "  1. Installed globally: cargo install --path ."
    echo "  2. Built: cargo build --release"
    echo "  3. Available in current directory"
    exit 1
fi

echo "Configuration:"
echo "  CuraLit Binary:   $CURALIT_BIN"
echo "  Checkpoint File:  $CHECKPOINT_FILE"
echo "  Collection Name:  $COLLECTION_NAME"
echo "  LLM Model:        $LLM_MODEL"
echo ""

# Step 1: Check if Qdrant is running
print_info "Checking Qdrant status..."
if curl -s http://localhost:6333/healthz > /dev/null 2>&1; then
    print_success "Qdrant is already running on port 6333"
else
    print_warning "Qdrant is not running. Starting Qdrant container..."
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Get absolute path for volume mount
    QDRANT_STORAGE_PATH="$(cd "$(dirname "$0")" && pwd)/qdrant_storage"
    
    # Check if Qdrant container already exists
    if docker ps -a --format '{{.Names}}' | grep -q '^curalit-qdrant$'; then
        print_info "Found existing Qdrant container..."
        
        # Check if it's running
        if docker ps --format '{{.Names}}' | grep -q '^curalit-qdrant$'; then
            print_warning "Container exists and is running, but health check failed. Restarting..."
            docker restart curalit-qdrant
        else
            # Container exists but is stopped - remove it and recreate with correct config
            print_info "Removing stopped container and recreating with correct configuration..."
            docker rm curalit-qdrant > /dev/null 2>&1 || true
            
            print_info "Creating new Qdrant container..."
            docker run -d \
                --name curalit-qdrant \
                -p 6333:6333 \
                -p 6334:6334 \
                -v "${QDRANT_STORAGE_PATH}:/qdrant/storage" \
                qdrant/qdrant
        fi
    else
        print_info "Creating new Qdrant container..."
        
        # Create storage directory if it doesn't exist
        mkdir -p "${QDRANT_STORAGE_PATH}"
        
        docker run -d \
            --name curalit-qdrant \
            -p 6333:6333 \
            -p 6334:6334 \
            -v "${QDRANT_STORAGE_PATH}:/qdrant/storage" \
            qdrant/qdrant
    fi
    
    # Wait for Qdrant to be ready
    print_info "Waiting for Qdrant to be ready..."
    for i in {1..30}; do
        if curl -s http://localhost:6333/healthz > /dev/null 2>&1; then
            print_success "Qdrant is ready!"
            break
        fi
        if [ $i -eq 30 ]; then
            print_error "Qdrant failed to start. Please check Docker logs:"
            echo "  docker logs curalit-qdrant"
            echo ""
            print_info "Debug information:"
            echo "  Storage path: ${QDRANT_STORAGE_PATH}"
            docker ps -a | grep curalit-qdrant || echo "  Container not found"
            exit 1
        fi
        sleep 1
    done
fi

# Step 2: Check if Ollama is running
print_info "Checking Ollama status..."
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    print_success "Ollama is running"
else
    print_error "Ollama is not running. Please start Ollama first:"
    echo "  - On Linux: systemctl start ollama"
    echo "  - On macOS: Start Ollama.app"
    echo "  - Or run: ollama serve"
    exit 1
fi

# Step 3: Check and pull embedding model
print_info "Checking embedding model (nomic-embed-text)..."
if ollama list | grep -q "nomic-embed-text"; then
    print_success "Embedding model already installed"
else
    print_warning "Embedding model not found. Pulling nomic-embed-text..."
    ollama pull nomic-embed-text
    print_success "Embedding model installed"
fi

# Ste$CURALIT_BINeck and pull LLM model (optional)
print_info "Checking LLM model ($LLM_MODEL)..."
if ollama list | grep -q "$LLM_MODEL"; then
    print_success "LLM model '$LLM_MODEL' is available"
else
    print_warning "LLM model '$LLM_MODEL' not found."
    read -p "Do you want to pull it now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Pulling model $LLM_MODEL (this may take a while)..."
        ollama pull "$LLM_MODEL"
        print_success "Model pulled successfully"
    else
        print_warning "Skipping model pull. You can pull it later with: ollama pull $LLM_MODEL"
    fi
fi

# Step 5: Build RAG index
print_header "Building RAG Index"
print_info "Processing articles from $CHECKPOINT_FILE..."

if ! $CURALIT_BIN rag-build -c "$CHECKPOINT_FILE" -n "$COLLECTION_NAME"; then
    print_error "Failed to build RAG index"
    echo ""
    print_warning "Common issues and solutions:"
    echo "  1. Qdrant connection error:"
    echo "     - Restart Qdrant: docker restart curalit-qdrant"
    echo "     - Check logs: docker logs curalit-qdrant"
    echo "     - Ensure port 6333 is not blocked"
    echo ""
    echo "  2. HTTP/2 protocol error:"
    echo "     - Stop and restart Qdrant:"
    echo "       docker stop curalit-qdrant"
    echo "       docker rm curalit-qdrant"
    echo "       ./rag_workflow.sh $CHECKPOINT_FILE $COLLECTION_NAME $LLM_MODEL"
    echo ""
    echo "  3. Embedding model issues:"
    echo "     - Verify Ollama is running: ollama list"
    echo "     - Re-pull model: ollama pull nomic-embed-text"
    echo ""
    
    # Offer to restart Qdrant
    read -p "Would you like to restart Qdrant and try again? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Restarting Qdrant..."
        docker restart curalit-qdrant
        sleep 5
        
        print_info "Retrying RAG index build..."
        if ! $CURALIT_BIN rag-build -c "$CHECKPOINT_FILE" -n "$COLLECTION_NAME"; then
            print_error "Failed again. Please check the error messages above."
            exit 1
        fi
    else
        exit 1
    fi
fi

print_success "RAG index built successfully!"

# Step 6: Interactive query mode
print_header "RAG Query Interface"

echo "You can now query your knowledge base in two ways:"
echo ""
echo "  1. Quick Query (retrieve relevant passages only):"
echo "     curalit rag-query -q \"your question\" -n $COLLECTION_NAME"
echo ""
echo "  2. Generate Answer (retrieve + LLM generation):"
echo "     curalit rag-generate -q \"your question\" -m $LLM_MODEL -n $COLLECTION_NAME"
echo ""

read -p "Would you like to try a query now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    read -p "Enter your question: " QUESTION
    
    if [ -z "$QUESTION" ]; then
        print_warning "No question provided. Exiting."
        exit 0
    fi
    
    echo ""
    print_info "Generating answer using RAG..."
    echo ""
    
    $CURALIT_BIN rag-generate -q "$QUESTION" -m "$LLM_MODEL" -n "$COLLECTION_NAME"
    
    echo ""
    print_success "Query complete!"
    echo ""
    echo "To run more queries, use:"
    echo "  curalit rag-generate -q \"your question\" -m $LLM_MODEL -n $COLLECTION_NAME"
fi

print_header "Workflow Complete"
print_success "RAG system is ready to use!"

echo ""
echo "Useful commands:"
echo "  - Query: curalit rag-query -q \"question\" -n $COLLECTION_NAME"
echo "  - Generate: curalit rag-generate -q \"question\" -m $LLM_MODEL -n $COLLECTION_NAME"
echo "  - Stop Qdrant: docker stop curalit-qdrant"
echo "  - View Qdrant UI: http://localhost:6333/dashboard"
echo ""

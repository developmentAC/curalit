#!/usr/bin/env bash

# Define color variables
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color (Reset)

# Use them with echo -e
# echo -e "${GREEN}SUCCESS:${NC} The operation completed successfully."
# echo -e "${YELLOW}WARNING:${NC} Disk space is low."
# echo -e "${BLUE}INFO:${NC} This is an informational message."
# echo -e "${RED}ERROR:${NC} Failed to connect to database."
    
echo -e "${BLUE}INFO:${NC} This script will run a quick test of CuraLit with QDrant"

echo -e  "${BLUE}INFO:${NC} Setup an empty Docker container for QDrant"

# echo building/starting Docker container
# file: readyDocker.sh
# echo create a new Docker contains for QDrant
# docker stop curalit-qdrant
# docker rm curalit-qdrant
# docker run -d --name curalit-qdrant -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage qdrant/qdrant

echo -e "${BLUE}INFO:${NC} Define keywords"
kw1="cancer"
kw2="immunotherapy"
results_dir="myResults/"
data_dir="./data/"

echo -e "${BLUE}INFO:${NC} Define data directories"

echo -e "${BLUE}INFO:${NC} Command to launch CuraLit"
./curalit search -k "$kw1" -k "$kw2" -d ./data -o "$results_dir"

echo -e "${BLUE}INFO:${NC} Command to build statistics report"
./curalit stats -c 0_out/"$results_dir"_*.csv

echo -e "${BLUE}INFO:${NC} Command to build RAG database"
./curalit rag-build -c 0_out/"$results_dir"_*.csv


echo -e "${BLUE}INFO:${NC} Command to build database"
./curalit db-build -k "$kw1" -k "$kw2" -d "$data_dir"

echo -e "${BLUE}INFO:${NC} Command to ask a question"
myQ="Describe a research project concerning immunotherapy. Please provide several articles to read on the subject. Comment on the articles and provide a summary of the research project. Please provide references to the articles."

echo -e "${YELLOW}INFO:${NC} Question to ask: {$myQ}"

./curalit rag-generate -m llama3 -q "$myQ"    



echo -e "${BLUE}INFO:${NC} Ask next question"

echo -e "${BLUE}INFO:${NC} Command to ask a question"
myQ="MY next question is: Describe a research project concerning immunotherapy. Please provide several articles to read on the subject. Comment on the articles and provide a summary of the research project. Please provide references to the articles."

echo -e "${YELLOW}INFO:${NC} Question to ask: {$myQ}"

./curalit rag-generate -m llama3 -q "$myQ"    


echo running parser
./target/release/curalit search -k "insulin resistance" -k "tau" -k "alzheimer" -d ../data --logic and -o results 

echo building/starting Docker container
echo create a new Docker contains for QDrant
docker stop curalit-qdrant
docker rm curalit-qdrant
docker run -d --name curalit-qdrant -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage qdrant/qdrant

echo building database
./target/release/curalit db-build -k "insulin resistance" -k "tau" -k "alzheimer" -d ../data

echo asking a question
./target/release/curalit rag-generate -m llama3 --use-db 0_out/curalit_*.db -q "Give me a short summary using data science to work with insulin data. Please provide a reference article" 

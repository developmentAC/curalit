echo create a new Docker contains for QDrant
docker stop curalit-qdrant
docker rm curalit-qdrant
docker run -d --name curalit-qdrant -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage qdrant/qdrant


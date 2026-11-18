# Running CuraLit 0.4.4 with Docker

These instructions run the CuraLit browser interface and CLI alongside Qdrant
and Ollama. You do not need to build CuraLit or install Rust or Python.

## Requirements

- Docker and the Docker Compose plugin.
- A Linux server or other Docker host that can run Linux amd64 images. The
  published `0.4.4` image currently supports amd64.
- Enough disk space for the images, PubMed data, results, and Ollama models.

## Start the services

Create a new directory and a Compose file:

```bash
mkdir -p curalit-docker
cd curalit-docker
mkdir -p data 0_out

cat > compose.yaml <<'EOF'
services:
  curalit:
    image: obonhamcarter/curalit:0.4.4
    ports:
      - "127.0.0.1:5050:5050"
    environment:
      OLLAMA_URL: http://ollama:11434
      QDRANT_URL: http://qdrant:6334
    volumes:
      - ./data:/app/data
      - ./0_out:/app/0_out
    depends_on:
      - qdrant
      - ollama
    restart: unless-stopped

  qdrant:
    image: qdrant/qdrant
    volumes:
      - qdrant_storage:/qdrant/storage
    restart: unless-stopped

  ollama:
    image: ollama/ollama
    volumes:
      - ollama_models:/root/.ollama
    restart: unless-stopped

volumes:
  qdrant_storage:
  ollama_models:
EOF

docker compose pull
docker compose up -d
```

The `cat` command overwrites `compose.yaml` if it already exists. Run subsequent
Compose commands from the same directory.

Open <http://localhost:5050> on the Docker host. Place extracted PubMed XML
files in `data/`; CuraLit saves results in `0_out/`. The container runs as UID
1000, which needs read access to `data/` and write access to `0_out/`. Adjust
host directory permissions if necessary.

## Download models for RAG

After starting the services, download an embedding model and a language model:

```bash
docker compose exec ollama ollama pull nomic-embed-text
docker compose exec ollama ollama pull llama3
```

Model downloads can take time and require additional disk space.

## Use the CLI interactively

Open a shell in the running CuraLit container:

```bash
docker compose exec curalit /bin/sh
curalit --help
```

Type `exit` to leave the shell.

## Access a remote server

From your own computer, open an SSH tunnel to the server:

```bash
ssh -L 5050:localhost:5050 user@your-server
```

Then open <http://localhost:5050> on your computer. The Compose file binds the
UI to the Docker host's loopback interface. The UI has no authentication
configured, so do not expose it publicly without appropriate access controls.

## Stop and restart

```bash
docker compose down
docker compose up -d
```

The `data/` and `0_out/` directories and the Qdrant and Ollama named volumes
remain available. Do not add `-v` to `docker compose down` unless you intend to
delete the named volumes, including downloaded models and Qdrant data.

## Troubleshooting

- **Image not found:** Confirm that the image is
  `obonhamcarter/curalit:0.4.4`, not the placeholder
  `your-dockerhub-user/curalit:latest`. If the repository is private, run
  `docker login` with an account that has access.
- **Permission denied:** Check the host permissions for `data/` and `0_out/`;
  the CuraLit container runs as UID 1000.
- **Service errors:** Inspect logs with `docker compose logs curalit qdrant ollama`.

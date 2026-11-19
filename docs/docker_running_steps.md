# CuraLit from Docker Hub: Quick Steps

Image on Docker Hub: `obonhamcarter/curalit:0.4.4` (linux/amd64)

This folder (`curalit-docker/`) holds everything needed to run it:

```
curalit-docker/
├── compose.yaml   # starts CuraLit + Qdrant (vector DB) + Ollama (LLMs)
├── data/          # put PubMed .xml files here  -> /app/data in the container
└── 0_out/         # results appear here          -> /app/0_out in the container
```

Always run the `docker compose` commands from inside this folder.

## 1. Start it

Open Docker Desktop first, then:

```bash
cd ~/Desktop/aa/DockerBusiness_curalit/curalit-docker
docker compose pull      # get the latest images from Docker Hub
docker compose up -d     # start all three services in the background
docker compose ps        # check: curalit should say "(healthy)"
```

## 2. Use the browser interface

Open <http://localhost:5050>

## 3. Use the command line (CLI) inside the container

```bash
docker compose exec curalit /bin/sh    # opens a shell inside the container
cd /app
curalit --help
curalit search -k study -k protein -d ./data --logic or -o myrun
exit                                   # leave the container
```

Or run a single command without opening a shell:

```bash
docker compose exec curalit curalit --help
```

Results written to `/app/0_out` in the container appear in `0_out/` on your Mac.

## 4. (First time only) Download the AI models for RAG

These are large (several GB) and only need to be downloaded once. They stay in a Docker volume.

```bash
docker compose exec ollama ollama pull nomic-embed-text
docker compose exec ollama ollama pull llama3
```

## 5. Stop it

```bash
docker compose down        # stops everything; data, results and models are kept
```

Do NOT use `docker compose down -v` unless you want to delete the downloaded
models and the Qdrant database.

## Starting from scratch on a new computer

You need only Docker Desktop. Make a folder, paste `compose.yaml` into it
(copy it from this folder, or from `curalit/docs/docker-usage.md`), then:

```bash
mkdir -p data 0_out
docker compose pull
docker compose up -d
```

## Handy commands

| What | Command |
|---|---|
| See logs | `docker compose logs -f curalit` |
| Restart | `docker compose restart` |
| Is it running? | `docker compose ps` |
| Try the sample data | `cp ../curalit/sample_data/synthetic_pubmed_sample.xml data/` |

## Publishing a new version (maintainer)

From the `curalit/` repository folder:

```bash
docker build -t obonhamcarter/curalit:0.4.5 -t obonhamcarter/curalit:latest .
docker login
docker push obonhamcarter/curalit:0.4.5
docker push obonhamcarter/curalit:latest
```

Then change the version tag in `compose.yaml` and run `docker compose pull && docker compose up -d`.

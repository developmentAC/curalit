# Running CuraLit with Docker: Step by Step

Image on Docker Hub: `obonhamcarter/curalit:0.4.4` (linux/amd64)

With Docker you do not need to install Rust, Python, Ollama or Qdrant. You only
need [Docker Desktop](https://www.docker.com/products/docker-desktop/) (or Docker
Engine with the Compose plugin).

## How the pieces fit together

CuraLit runs as three containers that talk to each other over Docker's
internal network. They are defined in [docker-compose.yml](../docker-compose.yml):

| Container | Image | What it does |
|---|---|---|
| `curalit` | `obonhamcarter/curalit:0.4.4` | The CuraLit command line and browser interface |
| `ollama` | `ollama/ollama` | Ollama, which runs the AI models (LLMs and embeddings) |
| `qdrant` | `qdrant/qdrant` | Vector database used by the RAG commands |

Ollama is **not** installed inside the CuraLit container; it runs in its own
container. CuraLit reaches it at `http://ollama:11434` (set by `OLLAMA_URL`).
This keeps the CuraLit image small and lets each piece be updated separately.

Folders in the repository that are shared with the CuraLit container:

```text
curalit/
├── docker-compose.yml   # defines the three containers
├── start_curalit.sh     # one-step startup script
├── data/                # put PubMed .xml files here  -> /app/data in the container
└── 0_out/               # results appear here          -> /app/0_out in the container
```

Run all commands below from the `curalit/` folder.

## 1. Start CuraLit (easiest way)

Open Docker Desktop first, then:

```bash
git clone https://github.com/developmentAC/curalit.git
cd curalit
./start_curalit.sh
```

The script:

1. checks that Docker is installed and running,
2. creates `data/` and `0_out/`, copying in a small sample file if `data/` is empty,
3. downloads and starts the CuraLit, Qdrant and Ollama containers,
4. downloads the Ollama models `nomic-embed-text` and `llama3` (about 5 GB, first run only),
5. runs a demo search into `0_out/getting_started/` and opens <http://localhost:5050>.

Later starts are fast because the images and models are already downloaded.

Other script options:

| Command | What it does |
|---|---|
| `./start_curalit.sh shell` | Open a command line inside the CuraLit container |
| `./start_curalit.sh status` | Show which services are running |
| `./start_curalit.sh logs` | Follow the CuraLit logs |
| `./start_curalit.sh stop` | Stop everything; data, results and models are kept |
| `./start_curalit.sh --skip-models` | Start without downloading the Ollama models |
| `./start_curalit.sh --help` | Show all options |

### Starting without the script

```bash
mkdir -p data 0_out
docker compose pull      # get the latest images from Docker Hub
docker compose up -d     # start all three services in the background
docker compose ps        # check: curalit should say "(healthy)"
```

Then download the models as shown in step 4.

## 2. Use the browser interface

Open <http://localhost:5050>. Every CuraLit command (search, stats, model
generation, RAG, database build) is available as a form.

## 3. Use the command line (CLI) inside the container

```bash
./start_curalit.sh shell        # or: docker compose exec -w /app curalit /bin/sh
curalit --help
curalit search -k methanol -k poisoning -d ./data -o myrun
curalit rag-build -c 0_out/myrun/results.csv
curalit rag-generate -q "What treatments are used for methanol poisoning?" -m llama3
exit                            # leave the container
```

Or run a single command without opening a shell:

```bash
docker compose exec -w /app curalit curalit --help
```

Results written to `/app/0_out` in the container appear in `0_out/` on your computer.

## 4. Ollama and its models

The Ollama program comes with the `ollama/ollama` image, so `docker compose pull`
(or `./start_curalit.sh`) installs it. Nothing needs to be installed on your
computer.

CuraLit's RAG commands need two models:

| Model | Size | Used for |
|---|---|---|
| `nomic-embed-text` | ~270 MB | Turning article text into vectors (`rag-build`, `rag-query`) |
| `llama3` | ~4.7 GB | Writing answers (`rag-generate`, `generate -b llama3`) |

`./start_curalit.sh` downloads both automatically. To manage models yourself,
run `ollama` inside the Ollama container:

```bash
docker compose exec ollama ollama pull nomic-embed-text   # download a model
docker compose exec ollama ollama pull llama3
docker compose exec ollama ollama list                    # see downloaded models
docker compose exec ollama ollama rm llama3               # remove a model
```

Models are stored in the Docker volume `ollama_models`, so they are kept when
the containers stop, restart or are updated. You only download them once.

To use a different model, download it and pass its name with `-m`:

```bash
docker compose exec ollama ollama pull mistral
docker compose exec -w /app curalit curalit rag-generate -q "your question" -m mistral
```

You can also have the script download different models:

```bash
CURALIT_MODELS="nomic-embed-text mistral" ./start_curalit.sh
```

### Optional: use Ollama installed directly on your computer

If you already run Ollama outside Docker (from <https://ollama.com>), CuraLit
can use it instead of the Ollama container:

```bash
ollama pull nomic-embed-text
ollama pull llama3
```

Then set `OLLAMA_URL` for the `curalit` service in `docker-compose.yml` to
`http://host.docker.internal:11434` and run `docker compose up -d`.

This matters most on Apple Silicon Macs: Ollama inside Docker on macOS can only
use the CPU, while Ollama installed directly on the Mac can use the GPU and
answers much faster.

## 5. Add your own PubMed data

Download `.xml.gz` files from the
[PubMed FTP archives](https://ftp.ncbi.nlm.nih.gov/pubmed/), extract them with
`gunzip`, and place the `.xml` files in `data/`. They are immediately visible to
the container as `/app/data`.

## 6. Stop it

```bash
./start_curalit.sh stop    # or: docker compose down
```

Data, results, the Qdrant database and the downloaded models are kept.

Do NOT use `docker compose down -v` unless you want to delete the downloaded
models and the Qdrant database.

## Running on a computer without the repository

You only need Docker and a compose file. Make a folder, create `compose.yaml`
in it (copy [docker-compose.yml](../docker-compose.yml), or the version in
[docker-usage.md](docker-usage.md)), then:

```bash
mkdir -p data 0_out
docker compose pull
docker compose up -d
docker compose exec ollama ollama pull nomic-embed-text
docker compose exec ollama ollama pull llama3
```

## Handy commands

| What | Command |
|---|---|
| Is it running? | `docker compose ps` |
| See logs | `docker compose logs -f curalit` |
| Restart | `docker compose restart` |
| Update to the newest images | `docker compose pull && docker compose up -d` |
| Try the sample data | `cp sample_data/synthetic_pubmed_sample.xml data/` |

## Troubleshooting

- **"Docker is not running"**: start Docker Desktop and wait until it says it is running.
- **Port 5050 already in use**: another CuraLit stack or app is using it. Stop it, or check with `docker ps`.
- **RAG commands fail or say a model is missing**: check `docker compose exec ollama ollama list` and pull the missing model.
- **Answers are very slow**: Ollama in Docker on macOS runs on the CPU only. See the optional native Ollama setup in step 4.
- **Permission denied on `0_out/` (Linux)**: the container runs as user ID 1000 and needs write access to `0_out/`.
- **Apple Silicon Mac**: the image is built for amd64 and runs through emulation, which is slower.

More details, including remote servers and SSH tunnels: [docker-usage.md](docker-usage.md).

## Publishing a new version (maintainer)

From the `curalit/` repository folder:

```bash
docker build -t obonhamcarter/curalit:0.4.5 -t obonhamcarter/curalit:latest .
docker login
docker push obonhamcarter/curalit:0.4.5
docker push obonhamcarter/curalit:latest
```

Then update the version tag in `docker-compose.yml` (and in
[docker-usage.md](docker-usage.md)) and run
`docker compose pull && docker compose up -d`.

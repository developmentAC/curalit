#!/usr/bin/env bash
# start_curalit.sh — start CuraLit with Docker and get ready for analysis.
#
#   ./start_curalit.sh                 start everything, download models, run a demo search
#   ./start_curalit.sh --skip-models   start without downloading Ollama models
#   ./start_curalit.sh shell           open a command line inside the CuraLit container
#   ./start_curalit.sh status          show what is running
#   ./start_curalit.sh logs            follow the CuraLit logs
#   ./start_curalit.sh stop            stop everything (data, results and models are kept)
#
# Models to download can be changed with:  CURALIT_MODELS="nomic-embed-text llama3" ./start_curalit.sh
# The CuraLit image can be changed with:   CURALIT_IMAGE=obonhamcarter/curalit:0.4.4 ./start_curalit.sh

set -euo pipefail

cd "$(dirname "$0")"

MODELS="${CURALIT_MODELS:-nomic-embed-text llama3}"
URL="http://localhost:5050"

info() { printf '\033[1;36m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m ✓ \033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m ! \033[0m %s\n' "$*"; }
die()  { printf '\033[1;31m ✗ \033[0m %s\n' "$*" >&2; exit 1; }

check_docker() {
    command -v docker >/dev/null 2>&1 \
        || die "Docker is not installed. Install Docker Desktop: https://www.docker.com/products/docker-desktop/"
    docker info >/dev/null 2>&1 \
        || die "Docker is installed but not running. Start Docker Desktop and try again."
    docker compose version >/dev/null 2>&1 \
        || die "The 'docker compose' plugin is missing. Update Docker Desktop or install docker-compose-plugin."
}

wait_for() {  # wait_for <description> <seconds> <command...>
    local what=$1 secs=$2; shift 2
    printf '    waiting for %s ' "$what"
    for _ in $(seq "$secs"); do
        if "$@" >/dev/null 2>&1; then echo; return 0; fi
        printf '.'; sleep 1
    done
    echo; return 1
}

curalit_healthy() {
    [ "$(docker inspect -f '{{.State.Health.Status}}' "$(docker compose ps -q curalit)")" = healthy ]
}

start() {
    local skip_models=$1
    check_docker
    ok "Docker is running"

    # Folders shared with the container
    mkdir -p data 0_out
    if [ "$(uname)" = Linux ] && [ "$(id -u)" != 1000 ]; then
        # The container runs as UID 1000 and must be able to write results.
        chmod a+rwX 0_out
    fi
    if ! ls data/*.xml >/dev/null 2>&1; then
        cp sample_data/*.xml data/
        warn "data/ had no PubMed XML files, so the sample file was copied in."
        warn "Replace it with real PubMed .xml files (https://ftp.ncbi.nlm.nih.gov/pubmed/) when ready."
    fi
    ok "data/ ($(ls data/*.xml | wc -l | tr -d ' ') XML file(s)) and 0_out/ are ready"

    info "Downloading images from Docker Hub (CuraLit, Qdrant, Ollama)…"
    docker compose pull
    info "Starting services…"
    docker compose up -d

    wait_for "CuraLit" 120 curalit_healthy || die "CuraLit did not start. See: docker compose logs curalit"
    ok "CuraLit is up at $URL"
    wait_for "Ollama" 60 docker compose exec -T ollama ollama list || die "Ollama did not start. See: docker compose logs ollama"
    ok "Ollama is up"

    if [ "$skip_models" = yes ]; then
        warn "Skipping model downloads (RAG questions need: $MODELS)"
    else
        for m in $MODELS; do
            if docker compose exec -T ollama ollama list | awk 'NR>1{print $1}' | grep -q "^$m\(:\|$\)"; then
                ok "Model $m already downloaded"
            else
                info "Downloading model $m (this can take a while the first time)…"
                docker compose exec -T ollama ollama pull "$m"
                ok "Model $m downloaded"
            fi
        done
    fi

    info "Running a quick demo search on data/ …"
    docker compose exec -T -w /app curalit \
        curalit search -k protein -k study --logic or -d ./data -o getting_started >/dev/null \
        && ok "Demo results written to 0_out/getting_started/ (open report.md there)" \
        || warn "Demo search failed; try it yourself with: ./start_curalit.sh shell"

    cat <<EOF

────────────────────────────────────────────────────────────
 CuraLit is ready.

 Browser:      $URL
 Your data:    $(pwd)/data      (PubMed .xml files)
 Your results: $(pwd)/0_out

 Command line: ./start_curalit.sh shell     then e.g.
               curalit search -k methanol -k poisoning -d ./data -o myrun
               curalit rag-build -c 0_out/myrun/results.csv
               curalit rag-generate -q "your question" -m llama3
 Stop:         ./start_curalit.sh stop
────────────────────────────────────────────────────────────
EOF
    if [ "$(uname)" = Darwin ]; then open "$URL" || true; fi
}

case "${1:-start}" in
    start|"")       start no ;;
    --skip-models)  start yes ;;
    shell)          check_docker; docker compose exec -w /app curalit /bin/sh ;;
    status)         check_docker; docker compose ps ;;
    logs)           check_docker; docker compose logs -f curalit ;;
    stop)           check_docker; docker compose down; ok "Stopped. Data, results and models are kept." ;;
    -h|--help|help) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//' ;;
    *)              die "Unknown option '$1'. Try: ./start_curalit.sh --help" ;;
esac

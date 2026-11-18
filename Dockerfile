# syntax=docker/dockerfile:1

# ---- Stage 1: build the Rust CLI ----
FROM rust:1-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev protobuf-compiler build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin curalit

# ---- Stage 2: runtime (Python web UI + CLI) ----
FROM python:3.14-slim-bookworm
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 curalit

WORKDIR /app
COPY pyproject.toml uv.lock .python-version README.md ./
RUN uv sync --frozen --no-install-project --no-dev \
    && uv pip install --python .venv/bin/python gunicorn

COPY webui ./webui
COPY src/network_template.py ./src/network_template.py
COPY sample_data ./sample_data
COPY --from=builder /build/target/release/curalit /usr/local/bin/curalit

RUN mkdir -p /app/data /app/0_out && chown -R curalit:curalit /app
USER curalit

ENV PATH="/app/.venv/bin:${PATH}" \
    CURALIT_BIN=/usr/local/bin/curalit \
    OLLAMA_URL=http://ollama:11434 \
    QDRANT_URL=http://qdrant:6334 \
    PYTHONUNBUFFERED=1

VOLUME ["/app/data", "/app/0_out"]
EXPOSE 5050

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s \
    CMD curl -fs http://localhost:5050/ >/dev/null || exit 1

# Web UI by default; run the CLI with: docker run ... <image> curalit --help
CMD ["gunicorn", "--bind", "0.0.0.0:5050", "--workers", "2", "--timeout", "1900", "--chdir", "webui", "app:app"]

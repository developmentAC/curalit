"""CuraLit Web UI

A small Flask app that exposes CuraLit's CLI functionality (search, stats,
generate, RAG build/query/generate, and database build) through a browser
form. It works by shelling out to the compiled `curalit` binary, so it
requires no changes to the Rust codebase logic itself.

Run with:
    uv run webui/app.py
or:
    python webui/app.py
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

from flask import Flask, render_template, request

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = REPO_ROOT / "0_out"

app = Flask(__name__)
app.config["MAX_CONTENT_LENGTH"] = 10 * 1024 * 1024  # 10 MB keyword file upload cap


def find_curalit_binary() -> str:
    """Locate the compiled curalit binary, preferring release over debug."""
    env_bin = os.environ.get("CURALIT_BIN")
    if env_bin and Path(env_bin).exists():
        return env_bin
    for candidate in (
        REPO_ROOT / "target" / "release" / "curalit",
        REPO_ROOT / "target" / "debug" / "curalit",
    ):
        if candidate.exists():
            return str(candidate)
    return "curalit"  # fall back to PATH


CURALIT_BIN = find_curalit_binary()


def run_command(args: list[str]) -> dict:
    """Run a curalit subcommand and capture its output."""
    cmd = [CURALIT_BIN, *args]
    try:
        result = subprocess.run(
            cmd,
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=1800,
        )
        return {
            "command": " ".join(cmd),
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }
    except FileNotFoundError:
        return {
            "command": " ".join(cmd),
            "returncode": -1,
            "stdout": "",
            "stderr": (
                f"Could not find the curalit binary at '{CURALIT_BIN}'. "
                "Build it first with: cargo build --release"
            ),
        }
    except subprocess.TimeoutExpired:
        return {
            "command": " ".join(cmd),
            "returncode": -1,
            "stdout": "",
            "stderr": "Command timed out after 30 minutes.",
        }


def keywords_from_form(form, files) -> list[str]:
    """Collect keywords typed in a text field (comma/newline separated)."""
    raw = form.get("keywords", "")
    keywords = [k.strip() for k in raw.replace(",", "\n").splitlines() if k.strip()]
    return keywords


def save_keywords_file(files) -> str | None:
    """Save an uploaded keywords file and return its path, if provided."""
    upload = files.get("keywords_file")
    if upload and upload.filename:
        dest_dir = OUT_DIR / "uploads"
        dest_dir.mkdir(parents=True, exist_ok=True)
        dest = dest_dir / upload.filename
        upload.save(dest)
        return str(dest)
    return None


def list_run_reports() -> list[dict]:
    """Find report.md files under 0_out/ for display in the UI."""
    reports = []
    if OUT_DIR.exists():
        for report in sorted(OUT_DIR.glob("*/report.md")):
            reports.append({"name": report.parent.name, "path": str(report)})
        for report in sorted(OUT_DIR.glob("*_report.md")):
            reports.append({"name": report.stem, "path": str(report)})
    return reports


@app.route("/")
def index():
    return render_template("index.html", reports=list_run_reports(), curalit_bin=CURALIT_BIN)


@app.route("/run/search", methods=["POST"])
def run_search():
    keywords = keywords_from_form(request.form, request.files)
    keywords_file = save_keywords_file(request.files)

    args = ["search"]
    for kw in keywords:
        args += ["-k", kw]
    if keywords_file:
        args += ["-f", keywords_file]
    args += ["-d", request.form.get("data_dir", "./data")]
    args += ["-o", request.form.get("output", "results")]
    args += ["-l", request.form.get("logic", "and")]
    args += ["-t", request.form.get("threshold", "1000")]
    if request.form.get("resume"):
        args += ["-r"]

    result = run_command(args)
    return render_template("result.html", title="Search", result=result, reports=list_run_reports())


@app.route("/run/stats", methods=["POST"])
def run_stats():
    checkpoint = request.form.get("checkpoint_file", "")
    result = run_command(["stats", "-c", checkpoint])
    return render_template("result.html", title="Stats", result=result, reports=list_run_reports())


@app.route("/run/generate", methods=["POST"])
def run_generate():
    args = [
        "generate",
        "-c", request.form.get("checkpoint_file", ""),
        "-m", request.form.get("model_name", ""),
        "-b", request.form.get("base_model", "llama3"),
    ]
    if request.form.get("package"):
        args += ["-p"]
    result = run_command(args)
    return render_template("result.html", title="Generate Model", result=result, reports=list_run_reports())


@app.route("/run/db-build", methods=["POST"])
def run_db_build():
    keywords = keywords_from_form(request.form, request.files)
    keywords_file = save_keywords_file(request.files)

    args = ["db-build"]
    for kw in keywords:
        args += ["-k", kw]
    if keywords_file:
        args += ["-f", keywords_file]
    args += ["-d", request.form.get("data_dir", "./data")]
    args += ["-n", request.form.get("db_name", "curalit")]
    args += ["-l", request.form.get("logic", "and")]

    result = run_command(args)
    return render_template("result.html", title="Database Build", result=result, reports=list_run_reports())


@app.route("/run/rag-build", methods=["POST"])
def run_rag_build():
    args = [
        "rag-build",
        "-c", request.form.get("checkpoint_file", ""),
        "-e", request.form.get("embedding_model", "nomic-embed-text"),
        "-n", request.form.get("collection_name", "curalit_articles"),
    ]
    result = run_command(args)
    return render_template("result.html", title="RAG Build", result=result, reports=list_run_reports())


@app.route("/run/rag-generate", methods=["POST"])
def run_rag_generate():
    args = [
        "rag-generate",
        "-q", request.form.get("query", ""),
        "-m", request.form.get("model", "llama3"),
        "-n", request.form.get("collection_name", "curalit_articles"),
        "-e", request.form.get("embedding_model", "nomic-embed-text"),
        "-k", request.form.get("top_k", "5"),
    ]
    use_db = request.form.get("use_db", "").strip()
    if use_db:
        args += ["--use-db", use_db]

    result = run_command(args)
    return render_template("result.html", title="Ask a Question (RAG Generate)", result=result, reports=list_run_reports())


@app.route("/report")
def view_report():
    path = request.args.get("path", "")
    content = ""
    if path and Path(path).resolve().is_relative_to(REPO_ROOT) and Path(path).exists():
        content = Path(path).read_text()
    return render_template("report.html", content=content, path=path)


if __name__ == "__main__":
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    app.run(host="127.0.0.1", port=5050, debug=True)

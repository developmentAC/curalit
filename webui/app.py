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

import json
import os
import statistics
import subprocess
import threading
import time
import uuid
from pathlib import Path

from flask import Flask, abort, jsonify, redirect, render_template, request, url_for

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


JOBS: dict[str, dict] = {}
JOBS_LOCK = threading.Lock()
DURATIONS_FILE = OUT_DIR / ".webui_durations.json"


def load_durations() -> dict[str, list[float]]:
    try:
        return json.loads(DURATIONS_FILE.read_text())
    except (OSError, ValueError):
        return {}


def record_duration(title: str, seconds: float) -> None:
    with JOBS_LOCK:
        data = load_durations()
        data[title] = (data.get(title, []) + [seconds])[-10:]
        try:
            DURATIONS_FILE.parent.mkdir(parents=True, exist_ok=True)
            DURATIONS_FILE.write_text(json.dumps(data))
        except OSError:
            pass


def _job_worker(job_id: str, args: list[str]) -> None:
    result = run_command(args)
    with JOBS_LOCK:
        job = JOBS[job_id]
        job["result"] = result
        job["finished"] = time.time()
        elapsed = job["finished"] - job["started"]
    if result["returncode"] == 0:
        record_duration(job["title"], elapsed)


def start_job(title: str, args: list[str]):
    """Run a command in the background and redirect to its progress page."""
    job_id = uuid.uuid4().hex
    history = load_durations().get(title, [])
    with JOBS_LOCK:
        JOBS[job_id] = {
            "title": title,
            "started": time.time(),
            "finished": None,
            "result": None,
            "estimate": statistics.median(history) if history else None,
        }
    threading.Thread(target=_job_worker, args=(job_id, args), daemon=True).start()
    return redirect(url_for("job_page", job_id=job_id))


def job_status(job_id: str) -> dict:
    with JOBS_LOCK:
        job = JOBS.get(job_id)
        if job is None:
            abort(404)
        job = dict(job)
    done = job["result"] is not None
    elapsed = (job["finished"] or time.time()) - job["started"]
    estimate = job["estimate"]
    if done:
        percent, remaining = 100.0, 0.0
    elif estimate:
        # Cap below 100% until the process actually exits.
        percent = min(elapsed / estimate * 100, 95.0)
        remaining = max(estimate - elapsed, 0.0)
    else:
        percent, remaining = None, None
    return {
        "done": done,
        "title": job["title"],
        "elapsed": elapsed,
        "percent": percent,
        "remaining": remaining,
        "estimated": estimate is not None,
    }


BROWSE_ROOTS = [Path.home().resolve(), REPO_ROOT.resolve()]


def _browse_allowed(path: Path) -> bool:
    return any(path.is_relative_to(root) for root in BROWSE_ROOTS)


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

    return start_job("Search", args)


@app.route("/run/stats", methods=["POST"])
def run_stats():
    checkpoint = request.form.get("checkpoint_file", "")
    return start_job("Stats", ["stats", "-c", checkpoint])


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
    return start_job("Generate Model", args)


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

    return start_job("Database Build", args)


@app.route("/run/rag-build", methods=["POST"])
def run_rag_build():
    args = [
        "rag-build",
        "-c", request.form.get("checkpoint_file", ""),
        "-e", request.form.get("embedding_model", "nomic-embed-text"),
        "-n", request.form.get("collection_name", "curalit_articles"),
    ]
    return start_job("RAG Build", args)


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

    return start_job("Ask a Question (RAG Generate)", args)


@app.route("/job/<job_id>")
def job_page(job_id):
    status = job_status(job_id)
    if status["done"]:
        with JOBS_LOCK:
            result = JOBS[job_id]["result"]
        return render_template("result.html", title=status["title"], result=result, reports=list_run_reports())
    return render_template("job.html", job_id=job_id, title=status["title"])


@app.route("/api/job/<job_id>")
def job_api(job_id):
    return jsonify(job_status(job_id))


@app.route("/api/browse")
def browse():
    raw = request.args.get("path", "")
    base = Path(raw).expanduser() if raw else REPO_ROOT
    if not base.is_absolute():
        base = REPO_ROOT / base
    path = base.resolve()
    if path.is_file():
        path = path.parent
    if not path.is_dir() or not _browse_allowed(path):
        return jsonify({"error": "Path not available"}), 400

    entries = []
    try:
        for child in sorted(path.iterdir(), key=lambda p: (not p.is_dir(), p.name.lower())):
            if child.name.startswith("."):
                continue
            entries.append({"name": child.name, "is_dir": child.is_dir(), "path": str(child)})
    except PermissionError:
        return jsonify({"error": "Permission denied"}), 403
    parent = path.parent if path.parent != path and _browse_allowed(path.parent) else None
    return jsonify({"path": str(path), "parent": str(parent) if parent else None, "entries": entries})


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

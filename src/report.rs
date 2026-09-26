//! Markdown reporting utilities and simplified run-directory naming.
//!
//! Every CuraLit command writes its output into a single, simply-named run
//! directory (e.g. `0_out/<name>/`) instead of scattering timestamped files
//! directly in `0_out/`. Each run directory also accumulates a human-readable
//! `report.md` summarizing what each command produced.

use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Resolve a simple, human-readable run directory under `base` for `name`.
///
/// If `reuse` is true and `base/name` already exists, it is reused as-is
/// (used for `--resume`). Otherwise, if `base/name` already exists, a
/// numeric suffix (`name_2`, `name_3`, ...) is appended until a free
/// directory is found, so previous runs are never overwritten silently.
pub fn resolve_run_dir(base: &Path, name: &str, reuse: bool) -> Result<PathBuf> {
    fs::create_dir_all(base).with_context(|| format!("Failed to create directory: {:?}", base))?;

    let candidate = base.join(name);
    if reuse || !candidate.exists() {
        fs::create_dir_all(&candidate)
            .with_context(|| format!("Failed to create run directory: {:?}", candidate))?;
        return Ok(candidate);
    }

    let mut n = 2;
    loop {
        let candidate = base.join(format!("{}_{}", name, n));
        if !candidate.exists() {
            fs::create_dir_all(&candidate)
                .with_context(|| format!("Failed to create run directory: {:?}", candidate))?;
            return Ok(candidate);
        }
        n += 1;
    }
}

/// Append a titled, timestamped Markdown section to `report.md` inside
/// `run_dir`, creating the file with a top-level heading if it is new.
pub fn append_report(run_dir: &Path, title: &str, body: &str) -> Result<PathBuf> {
    let report_path = run_dir.join("report.md");

    let is_new = !report_path.exists();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&report_path)
        .with_context(|| format!("Failed to open report file: {:?}", report_path))?;

    if is_new {
        writeln!(file, "# CuraLit Run Report\n")?;
    }

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "## {} ({})\n", title, now)?;
    writeln!(file, "{}\n", body.trim_end())?;

    Ok(report_path)
}

/// Append a titled, timestamped Markdown section to an arbitrary report file
/// (used by RAG commands, which are not tied to a single run directory).
pub fn append_report_file(report_path: &Path, title: &str, body: &str) -> Result<()> {
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    let is_new = !report_path.exists();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(report_path)
        .with_context(|| format!("Failed to open report file: {:?}", report_path))?;

    if is_new {
        writeln!(file, "# CuraLit RAG Report\n")?;
    }

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "## {} ({})\n", title, now)?;
    writeln!(file, "{}\n", body.trim_end())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("curalit_report_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn test_resolve_run_dir_creates_clean_directory() {
        let base = temp_dir("clean");
        let run_dir = resolve_run_dir(&base, "myresults", false).unwrap();
        assert_eq!(run_dir, base.join("myresults"));
        assert!(run_dir.is_dir());
        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn test_resolve_run_dir_avoids_collision() {
        let base = temp_dir("collision");
        let first = resolve_run_dir(&base, "run", false).unwrap();
        let second = resolve_run_dir(&base, "run", false).unwrap();
        let third = resolve_run_dir(&base, "run", false).unwrap();

        assert_eq!(first, base.join("run"));
        assert_eq!(second, base.join("run_2"));
        assert_eq!(third, base.join("run_3"));
        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn test_resolve_run_dir_reuses_when_resuming() {
        let base = temp_dir("resume");
        let first = resolve_run_dir(&base, "run", false).unwrap();
        fs::write(first.join("marker.txt"), "keep me").unwrap();

        let resumed = resolve_run_dir(&base, "run", true).unwrap();
        assert_eq!(resumed, first);
        assert!(resumed.join("marker.txt").exists());
        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn test_append_report_creates_and_appends_sections() {
        let base = temp_dir("report");
        fs::create_dir_all(&base).unwrap();

        let report_path = append_report(&base, "Search", "- Articles matched: 5").unwrap();
        assert!(report_path.exists());

        append_report(&base, "Statistics", "- Total articles: 5").unwrap();

        let content = fs::read_to_string(&report_path).unwrap();
        assert!(content.contains("# CuraLit Run Report"));
        assert!(content.contains("## Search"));
        assert!(content.contains("Articles matched: 5"));
        assert!(content.contains("## Statistics"));
        assert!(content.contains("Total articles: 5"));
        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn test_append_report_file_creates_and_appends() {
        let base = temp_dir("report_file");
        let report_path = base.join("collection_report.md");

        append_report_file(&report_path, "RAG Query", "**Question:** test\n").unwrap();
        append_report_file(&report_path, "RAG Generate", "**Answer:** ok\n").unwrap();

        let content = fs::read_to_string(&report_path).unwrap();
        assert!(content.contains("# CuraLit RAG Report"));
        assert!(content.contains("## RAG Query"));
        assert!(content.contains("## RAG Generate"));
        fs::remove_dir_all(&base).ok();
    }
}

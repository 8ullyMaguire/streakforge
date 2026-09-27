// Manifesto content — serves curated markdown docs from a local directory.
// GET /api/manifesto           -> list of docs (id, title, filename)
// GET /api/manifesto/{id}      -> raw markdown body
// The directory is configurable via MANIFESTOS_DIR (default ./manifestos).

use crate::error::{ApiError, ApiResult};
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use std::path::{Path as FsPath, PathBuf};

#[derive(Debug, Serialize)]
pub struct ManifestoContent {
    pub id: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ManifestoDoc {
    pub id: String,
    pub title: String,
    pub filename: String,
}

#[derive(Debug, Serialize)]
pub struct ManifestoList {
    pub docs: Vec<ManifestoDoc>,
}

fn manifestos_dir(state: &AppState) -> PathBuf {
    FsPath::new(&state.cfg.manifestos_dir).to_path_buf()
}

fn title_from_filename(filename: &str) -> String {
    // "01_starting_guide.md" -> "Starting Guide"
    //
    // The numeric prefix is stripped generically rather than by listing each
    // known prefix: a hardcoded trim_start_matches("11_") chain meant any doc
    // added past the tenth kept its number in the sidebar, and the omission is
    // invisible until the doc is deployed. Any leading digits and one underscore
    // go, so the numbering can grow without a code change.
    let stem = filename
        .trim_end_matches(".md")
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '_')
        .replace('_', " ");
    stem.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect()
}

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<ManifestoList>> {
    let dir = manifestos_dir(&state);
    let mut docs = Vec::new();
    let entries = std::fs::read_dir(&dir)
        .map_err(|e| ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Cannot read manifestos dir: {e}")))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_string();
        let id = filename.trim_end_matches(".md").to_string();
        let title = title_from_filename(&filename);
        docs.push(ManifestoDoc { id, title, filename });
    }
    docs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Json(ManifestoList { docs }))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<ManifestoContent>> {
    // sanitize: only allow [a-zA-Z0-9_-] and .md extension
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(ApiError::bad_request("Invalid manifesto id"));
    }
    let path = manifestos_dir(&state).join(format!("{id}.md"));
    let body = std::fs::read_to_string(&path)
        .map_err(|_| ApiError::not_found("Manifesto not found"))?;
    Ok(Json(ManifestoContent { id, content: body }))
}

#[cfg(test)]
mod tests {
    use super::title_from_filename;

    // The prefix trim used to be a hardcoded chain of trim_start_matches calls,
    // one per known doc. Adding an 11th doc rendered as "11 roadmap" in the
    // doctrine sidebar, and nothing failed — the only symptom was a stray number
    // in a UI nobody tests. These pin the generic behaviour instead.
    #[test]
    fn strips_the_numeric_prefix() {
        assert_eq!(title_from_filename("01_starting_guide.md"), "Starting guide");
        assert_eq!(title_from_filename("10_organizing_bnwo.md"), "Organizing bnwo");
    }

    #[test]
    fn strips_prefixes_past_ten() {
        assert_eq!(title_from_filename("11_roadmap.md"), "Roadmap");
        assert_eq!(title_from_filename("42_something.md"), "Something");
    }

    #[test]
    fn handles_a_bare_filename_and_odd_prefixes() {
        assert_eq!(title_from_filename("roadmap.md"), "Roadmap");
        // No digits: the whole name survives.
        assert_eq!(title_from_filename("appendix_b.md"), "Appendix b");
        // Leading underscores with no number are trimmed too, and must not panic
        // or produce an empty title.
        assert_eq!(title_from_filename("__roadmap.md"), "Roadmap");
    }
}

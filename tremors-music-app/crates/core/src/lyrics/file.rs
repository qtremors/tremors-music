// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Conflict-aware local lyric documents. Audio files are never modified here.
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};
#[derive(Clone, Debug)]
pub struct Document {
    pub song_id: i64,
    pub path: PathBuf,
    pub text: String,
    original: Option<Vec<u8>>,
}
fn existing(path: &Path) -> Result<Option<Vec<u8>>> {
    use std::io::Read;
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut bytes = Vec::new();
    file.take(2_000_001).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 2_000_000, "Lyrics exceed the 2 MB limit");
    Ok(Some(bytes))
}
pub fn read(song: &crate::model::Song) -> Result<Document> {
    let path = ["lrc", "ttml", "txt"]
        .into_iter()
        .map(|ext| song.path.with_extension(ext))
        .find(|p| p.is_file())
        .unwrap_or_else(|| song.path.with_extension("lrc"));
    let original = existing(&path)?;
    let text = original
        .as_ref()
        .map(|b| String::from_utf8(b.clone()))
        .transpose()
        .context("Lyrics must be UTF-8")?
        .unwrap_or_else(|| song.details.lyrics.clone());
    Ok(Document {
        song_id: song.id,
        path,
        text,
        original,
    })
}
pub fn save(document: &Document) -> Result<()> {
    ensure!(
        document
            .path
            .extension()
            .is_some_and(|e| e == "lrc" || e == "ttml" || e == "txt"),
        "Choose a lyric sidecar file"
    );
    ensure!(
        document.text.len() <= 2_000_000,
        "Lyrics exceed the 2 MB limit"
    );
    if document.path.extension().is_some_and(|e| e == "ttml") {
        roxmltree::Document::parse(&document.text).context("TTML is not valid XML")?;
    }
    ensure!(
        existing(&document.path)? == document.original,
        "This lyric file changed outside Tremors Music. Reopen the editor before saving."
    );
    crate::file_io::atomic_write(&document.path, document.text.as_bytes())
}

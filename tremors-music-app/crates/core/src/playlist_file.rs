// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Portable UTF-8 M3U playlists. Entries preserve order and duplicates.

use crate::{file_io::atomic_write, model::Song};
use anyhow::{Context, Result, bail, ensure};
use std::{
    io::Read,
    path::{Component, Path, PathBuf},
};

const MAX_PLAYLIST_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug)]
pub struct ImportedPlaylist {
    pub name: String,
    pub paths: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

pub fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(result.components().next_back(), Some(Component::Normal(_))) =>
            {
                result.pop();
            }
            Component::ParentDir if result.has_root() => {}
            _ => result.push(component.as_os_str()),
        }
    }
    result
}

pub fn read(path: &Path) -> Result<ImportedPlaylist> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_PLAYLIST_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_PLAYLIST_BYTES {
        bail!("Playlist exceeds the 8 MiB limit");
    }
    let text = std::str::from_utf8(&bytes)
        .context("Save this playlist as UTF-8 M3U/M3U8 before importing")?;
    let parent = dunce::canonicalize(
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?;
    let mut playlist = ImportedPlaylist {
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported playlist")
            .chars()
            .take(120)
            .collect(),
        paths: Vec::new(),
        warnings: Vec::new(),
    };
    for (index, line) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let entry = if line.starts_with("file:") {
            match url::Url::parse(line)
                .ok()
                .and_then(|url| url.to_file_path().ok())
            {
                Some(path) => path,
                None => {
                    playlist
                        .warnings
                        .push(format!("Line {}: unsupported file URL", index + 1));
                    continue;
                }
            }
        } else if line.contains("://") {
            playlist.warnings.push(format!(
                "Line {}: only local music files can be imported",
                index + 1
            ));
            continue;
        } else {
            #[cfg(not(windows))]
            if line.as_bytes().get(1) == Some(&b':') || line.starts_with("\\\\") {
                playlist.warnings.push(format!(
                    "Line {}: this path belongs to another operating system",
                    index + 1
                ));
                continue;
            }
            if parent.join(line).is_file() {
                PathBuf::from(line)
            } else {
                PathBuf::from(line.replace('\\', "/"))
            }
        };
        let entry = normalize(&if entry.is_absolute() {
            entry
        } else {
            parent.join(entry)
        });
        match dunce::canonicalize(&entry) {
            Ok(path) if path.is_file() => playlist.paths.push(path),
            _ => playlist
                .warnings
                .push(format!("Missing file: {}", entry.display())),
        }
    }
    if playlist.paths.is_empty() {
        bail!(
            "No readable local music files in this playlist. {}",
            playlist.warnings.first().cloned().unwrap_or_default()
        );
    }
    Ok(playlist)
}

fn relative_path(path: &Path, parent: &Path) -> Option<PathBuf> {
    let path: Vec<_> = path.components().collect();
    let base: Vec<_> = parent.components().collect();
    if path.first() != base.first() {
        return None;
    }
    let common = path.iter().zip(&base).take_while(|(a, b)| a == b).count();
    let mut relative = PathBuf::new();
    for _ in common..base.len() {
        relative.push("..");
    }
    for component in &path[common..] {
        relative.push(component.as_os_str());
    }
    Some(relative)
}

pub fn write(path: &Path, songs: &[Song], relative: bool) -> Result<()> {
    ensure!(
        path.extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| matches!(s.to_ascii_lowercase().as_str(), "m3u" | "m3u8")),
        "Choose a .m3u or .m3u8 playlist filename"
    );
    let parent = dunce::canonicalize(
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?;
    let mut content = String::from("#EXTM3U\n");
    for song in songs {
        let source = dunce::canonicalize(&song.path)
            .with_context(|| format!("Music file is unavailable: {}", song.path.display()))?;
        let exported = if relative {
            relative_path(&source, &parent).unwrap_or(source)
        } else {
            source
        };
        let name = format!("{} - {}", song.artist, song.title).replace(['\r', '\n'], " ");
        let entry = exported
            .to_str()
            .context("Playlist paths must be valid Unicode")?;
        if entry.contains(['\r', '\n']) {
            bail!("A music path contains a line break and cannot be exported");
        }
        // Prefix relative names beginning with # so they cannot turn into M3U directives.
        #[cfg(windows)]
        let entry = entry.replace('\\', "/");
        content.push_str(&format!(
            "#EXTINF:{},{}\n{}{}\n",
            song.duration.max(0.) as u64,
            name,
            if entry.starts_with('#') { "./" } else { "" },
            entry
        ));
    }
    atomic_write(path, content.as_bytes())
}

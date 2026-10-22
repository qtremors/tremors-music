// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Versioned curation archives. Track paths are portable identities; database IDs stay local.
use crate::{
    file_io,
    library::Library,
    settings::{AppPaths, Preferences},
};
use anyhow::{Context, Result, ensure};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

const MAX_BYTES: u64 = 32 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    pub version: u32,
    pub created_at: i64,
    pub roots: Vec<PathBuf>,
    pub preferences: Preferences,
    pub songs: Vec<TrackCuration>,
    pub playlists: Vec<SavedPlaylist>,
    pub artwork: Vec<SavedArtwork>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackCuration {
    pub path: PathBuf,
    pub favorite: bool,
    pub play_count: u32,
    pub last_played: Option<i64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedPlaylist {
    pub name: String,
    pub tracks: Vec<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedArtwork {
    pub artist: Option<String>,
    pub playlist: Option<usize>,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct Restore {
    pub backup: Backup,
    pub remap: Option<(PathBuf, PathBuf)>,
    pub preferences: bool,
}
impl Restore {
    pub fn restored_preferences(&self) -> Preferences {
        let mut preferences = self.backup.preferences.clone();
        preferences.excluded_folders = preferences
            .excluded_folders
            .iter()
            .map(|p| self.path(p))
            .collect();
        preferences.lyric_offsets = preferences
            .lyric_offsets
            .into_iter()
            .map(|(path, offset)| (self.path(&path), offset))
            .collect();
        preferences
    }
    pub fn path(&self, path: &Path) -> PathBuf {
        self.remap
            .as_ref()
            .and_then(|(old, new)| {
                path.strip_prefix(old)
                    .ok()
                    .map(|relative| new.join(relative))
            })
            .unwrap_or_else(|| path.to_path_buf())
    }
}

pub fn write(library: &Library, destination: &Path) -> Result<()> {
    ensure!(
        destination
            .extension()
            .is_some_and(|ext| ext == "tremors-backup"),
        "Choose a .tremors-backup file; music files cannot be overwritten."
    );
    let snapshot = library.full_snapshot()?;
    let by_id: HashMap<_, _> = snapshot.songs.iter().map(|s| (s.id, &s.path)).collect();
    let mut artwork = Vec::new();
    for (key, path) in &snapshot.artwork {
        let (artist, playlist) = if let Some(artist) = key.strip_prefix("artist:") {
            (Some(artist.to_owned()), None)
        } else if let Some(id) = key
            .strip_prefix("playlist:")
            .and_then(|id| id.parse::<i64>().ok())
        {
            (None, snapshot.playlists.iter().position(|p| p.id == id))
        } else {
            continue;
        };
        ensure!(
            std::fs::metadata(path)?.len() <= 8 * 1024 * 1024,
            "Custom artwork exceeds the backup size limit"
        );
        artwork.push(SavedArtwork {
            artist,
            playlist,
            bytes: std::fs::read(path)?,
        });
    }
    let archive = Backup {
        version: 1,
        created_at: crate::model::now(),
        roots: snapshot.roots,
        preferences: snapshot.preferences,
        songs: snapshot
            .songs
            .iter()
            .map(|s| TrackCuration {
                path: s.path.clone(),
                favorite: s.favorite,
                play_count: s.play_count,
                last_played: s.last_played,
            })
            .collect(),
        playlists: snapshot
            .playlists
            .iter()
            .map(|pl| SavedPlaylist {
                name: pl.name.clone(),
                tracks: pl
                    .entries
                    .iter()
                    .filter_map(|e| by_id.get(&e.song_id).map(|p| (*p).clone()))
                    .collect(),
            })
            .collect(),
        artwork,
    };
    let bytes = serde_json::to_vec(&archive)?;
    ensure!(
        bytes.len() as u64 <= MAX_BYTES,
        "Backup exceeds the 32 MiB limit"
    );
    file_io::atomic_write(destination, &bytes)
}

pub fn read(path: &Path) -> Result<Backup> {
    use std::io::Read;
    ensure!(
        std::fs::metadata(path)?.len() <= MAX_BYTES,
        "Backup exceeds the 32 MiB limit"
    );
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_BYTES,
        "Backup exceeds the 32 MiB limit"
    );
    let backup: Backup = serde_json::from_slice(&bytes).context("Invalid curation backup")?;
    validate(&backup)?;
    Ok(backup)
}
fn validate(backup: &Backup) -> Result<()> {
    ensure!(
        backup.version == 1,
        "Unsupported backup version {}",
        backup.version
    );
    ensure!(
        backup.songs.len() <= 200_000 && backup.playlists.len() <= 10_000,
        "Backup contains too many records"
    );
    for pl in &backup.playlists {
        ensure!(
            !pl.name.trim().is_empty() && pl.name.chars().count() <= 120,
            "Invalid playlist name in backup"
        );
    }
    for art in &backup.artwork {
        ensure!(
            art.bytes.len() <= 8 * 1024 * 1024 && art.artist.is_some() != art.playlist.is_some(),
            "Invalid artwork in backup"
        );
        if let Some(index) = art.playlist {
            ensure!(
                index < backup.playlists.len(),
                "Invalid artwork playlist reference"
            );
        }
    }
    Ok(())
}

pub fn restore(library: &mut Library, paths: &AppPaths, restore: &Restore) -> Result<Vec<String>> {
    validate(&restore.backup)?;
    let snapshot = library.full_snapshot()?;
    let by_path: HashMap<_, _> = snapshot.songs.iter().map(|s| (&s.path, s.id)).collect();
    let mut warnings = Vec::new();
    let matched = restore
        .backup
        .songs
        .iter()
        .filter(|s| by_path.contains_key(&restore.path(&s.path)))
        .count();
    ensure!(
        matched > 0 || restore.backup.songs.is_empty(),
        "No backup tracks match this library. Scan your music folders or adjust the folder mapping first."
    );
    let mut images = Vec::new();
    let mut image_bytes = 0;
    // Validate every image before changing either files or the database. Never use archive filenames.
    for art in &restore.backup.artwork {
        let mut reader =
            image::ImageReader::new(std::io::Cursor::new(&art.bytes)).with_guessed_format()?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(4096);
        limits.max_image_height = Some(4096);
        limits.max_alloc = Some(64 * 1024 * 1024);
        reader.limits(limits);
        let image = reader.decode().context("Invalid backup artwork")?;
        let mut png = std::io::Cursor::new(Vec::new());
        image.write_to(&mut png, image::ImageFormat::Png)?;
        image_bytes += png.get_ref().len();
        ensure!(
            image_bytes as u64 <= MAX_BYTES,
            "Restored artwork exceeds the 32 MiB limit"
        );
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        png.get_ref().hash(&mut hash);
        let path = paths.covers.join(format!("{:016x}.png", hash.finish()));
        images.push((art, path, png.into_inner()));
    }
    for (_, path, bytes) in &images {
        file_io::atomic_write(path, bytes)?;
    }
    let tx = library.connection.transaction()?;
    for song in &restore.backup.songs {
        if let Some(id) = by_path.get(&restore.path(&song.path)) {
            // Merging twice never doubles play counts or erases existing favorites.
            tx.execute("UPDATE songs SET favorite=MAX(favorite,?),play_count=MAX(play_count,?),last_played=CASE WHEN last_played IS NULL THEN ? WHEN ? IS NULL THEN last_played ELSE MAX(last_played,?) END WHERE id=?",params![song.favorite,song.play_count,song.last_played,song.last_played,song.last_played,id])?;
        } else {
            warnings.push(format!(
                "Not in library: {}",
                restore.path(&song.path).display()
            ));
        }
    }
    let mut playlists = Vec::new();
    for pl in &restore.backup.playlists {
        tx.execute("INSERT INTO playlists(name) VALUES(?)", [&pl.name])?;
        let id = tx.last_insert_rowid();
        playlists.push(id);
        for (position, path) in pl.tracks.iter().enumerate() {
            if let Some(song) = by_path.get(&restore.path(path)) {
                tx.execute(
                    "INSERT INTO playlist_entries(playlist_id,song_id,position) VALUES(?,?,?)",
                    params![id, song, position as i64],
                )?;
            }
        }
    }
    for (art, path, _) in images {
        let key = if let Some(artist) = &art.artist {
            format!("artist:{artist}")
        } else {
            format!("playlist:{}", playlists[art.playlist.unwrap()])
        };
        tx.execute("INSERT INTO artwork_overrides(key,cover) VALUES(?,?) ON CONFLICT(key) DO UPDATE SET cover=excluded.cover",params![key,path.to_string_lossy()])?;
    }
    if restore.preferences {
        tx.execute("INSERT INTO settings(key,value) VALUES('preferences',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[serde_json::to_string(&restore.restored_preferences())?])?;
    }
    tx.commit()?;
    Ok(warnings)
}

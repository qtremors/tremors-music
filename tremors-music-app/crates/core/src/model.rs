// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Song {
    pub id: i64,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: Option<u32>,
    pub track: Option<u32>,
    pub disc: Option<u32>,
    pub duration: f64,
    pub cover: Option<PathBuf>,
    pub favorite: bool,
    pub play_count: u32,
    pub added_at: i64,
    pub last_played: Option<i64>,
    pub details: TrackDetails,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TrackDetails {
    pub file_size: u64,
    pub modified_ns: u64,
    pub sidecar_modified_ns: u64,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub bit_depth: Option<u8>,
    pub format: String,
    pub lyrics: String,
    pub tags: std::collections::BTreeMap<String, String>,
}

impl Song {
    pub fn matches(&self, query: &str) -> bool {
        let text = format!(
            "{} {} {} {}",
            self.title, self.artist, self.album, self.genre
        )
        .to_lowercase();
        query
            .to_lowercase()
            .split_whitespace()
            .all(|word| text.contains(word))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaylistEntry {
    pub id: i64,
    pub song_id: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub entries: Vec<PlaylistEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub songs: Vec<Song>,
    pub roots: Vec<PathBuf>,
    pub playlists: Vec<Playlist>,
    pub preferences: crate::settings::Preferences,
    pub session: crate::queue::QueueState,
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn duration_label(seconds: f64) -> String {
    let seconds = seconds.max(0.) as u64;
    if seconds >= 3600 {
        format!(
            "{}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    } else {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}

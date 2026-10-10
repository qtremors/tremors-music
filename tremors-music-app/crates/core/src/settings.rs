// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn cycle(self) -> Self {
        match self {
            Self::Off => Self::All,
            Self::All => Self::One,
            Self::One => Self::Off,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub light_theme: bool,
    pub accent: String,
    pub show_artwork: bool,
    pub watch_folders: bool,
    pub replay_gain: bool,
    pub sort: SongSort,
    pub descending: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SongSort {
    #[default]
    Title,
    Artist,
    Album,
    Duration,
    Year,
    Added,
    Size,
}

impl SongSort {
    pub const ALL: [Self; 7] = [
        Self::Title,
        Self::Artist,
        Self::Album,
        Self::Duration,
        Self::Year,
        Self::Added,
        Self::Size,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::Album => "Album",
            Self::Duration => "Duration",
            Self::Year => "Year",
            Self::Added => "Date added",
            Self::Size => "File size",
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            volume: 0.7,
            shuffle: false,
            repeat: RepeatMode::Off,
            light_theme: false,
            accent: "Neutral".into(),
            show_artwork: true,
            watch_folders: true,
            replay_gain: false,
            sort: SongSort::Title,
            descending: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub data: PathBuf,
    pub database: PathBuf,
    pub covers: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let data = if let Some(path) = std::env::var_os("TREMORS_DATA_DIR") {
            PathBuf::from(path)
        } else {
            ProjectDirs::from("com", "Tremors", "Tremors Music")
                .context("Cannot locate the local application data directory")?
                .data_local_dir()
                .to_path_buf()
        };
        Self::at(data)
    }

    pub fn at(data: PathBuf) -> Result<Self> {
        let covers = data.join("covers");
        std::fs::create_dir_all(&covers)
            .with_context(|| format!("Cannot create {}", covers.display()))?;
        Ok(Self {
            database: data.join("library.sqlite3"),
            covers,
            data,
        })
    }
}

// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Edit the file's native tags on a sibling copy, preserving audio and unedited tag items.

use anyhow::{Result, ensure};
use lofty::{
    file::{TaggedFile, TaggedFileExt},
    tag::{ItemKey, Tag, TagItem},
};
pub use lofty::{
    picture::{Picture, PictureType},
    tag::TagType,
};
use std::path::PathBuf;

const COMMON: &[(&str, &str)] = &[
    ("TrackTitle", "Title"),
    ("TrackArtist", "Artist"),
    ("AlbumTitle", "Album"),
    ("AlbumArtist", "Album artist"),
    ("Genre", "Genre"),
    ("RecordingDate", "Release date / year"),
    ("OriginalReleaseDate", "Original release date"),
    ("TrackNumber", "Track number"),
    ("TrackTotal", "Total tracks"),
    ("DiscNumber", "Disc number"),
    ("DiscTotal", "Total discs"),
    ("Composer", "Composer"),
    ("Conductor", "Conductor"),
    ("Lyricist", "Lyricist"),
    ("Bpm", "BPM"),
    ("InitialKey", "Musical key"),
    ("Mood", "Mood"),
    ("Language", "Language"),
    ("Publisher", "Publisher"),
    ("CopyrightMessage", "Copyright"),
    ("Isrc", "ISRC"),
    ("CatalogNumber", "Catalog number"),
    ("Comment", "Comment"),
    ("ReplayGainTrackGain", "ReplayGain track gain"),
    ("ReplayGainTrackPeak", "ReplayGain track peak"),
    ("ReplayGainAlbumGain", "ReplayGain album gain"),
    ("ReplayGainAlbumPeak", "ReplayGain album peak"),
    ("Lyrics", "Embedded lyrics"),
    ("UnsyncLyrics", "Embedded lyrics"),
];

#[derive(Clone, Debug)]
pub struct MetadataField {
    pub key: ItemKey,
    pub label: String,
    pub value: String,
    original: Option<TagItem>,
}

#[derive(Clone, Debug)]
pub struct Artwork {
    pub picture: Picture,
    pub preview: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct MetadataDocument {
    pub song_id: i64,
    pub path: PathBuf,
    pub fields: Vec<MetadataField>,
    pub pictures: Vec<Artwork>,
    pub tag_type: TagType,
    pub has_sidecar_lyrics: bool,
    stamp: (u64, u64, u64),
    original_pictures: Vec<Picture>,
}

pub fn label(key: ItemKey) -> String {
    let name = format!("{key:?}");
    COMMON
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, label)| (*label).to_owned())
        .unwrap_or(name)
}

fn preferred_type(file: &TaggedFile) -> TagType {
    // ID3v2 supports rich metadata and embedded pictures in PCM WAV files as well.
    if file.file_type() == lofty::file::FileType::Wav {
        TagType::Id3v2
    } else {
        file.primary_tag_type()
    }
}

fn editing_tag(file: &TaggedFile) -> Tag {
    let kind = preferred_type(file);
    file.tag(kind).cloned().unwrap_or_else(|| {
        let mut tag = file.first_tag().cloned().unwrap_or_else(|| Tag::new(kind));
        tag.re_map(kind);
        tag
    })
}

impl MetadataDocument {
    pub fn add_field(&mut self, key: ItemKey) -> Result<()> {
        ensure!(
            key.map_key(self.tag_type).is_some(),
            "This field is not supported by the file's tag format"
        );
        self.fields.push(MetadataField {
            key,
            label: label(key),
            value: String::new(),
            original: None,
        });
        Ok(())
    }
    pub fn available_fields(&self) -> Vec<(ItemKey, String)> {
        ItemKey::supported_keys(self.tag_type)
            .iter()
            .map(|key| (*key, label(*key)))
            .collect()
    }
}

mod reader;
mod writer;
pub use reader::{load_artwork, read};
pub use writer::save;

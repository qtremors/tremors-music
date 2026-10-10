// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use crate::model::{Song, TrackDetails};
use anyhow::{Context, Result, bail};
use lofty::{
    file::{AudioFile, TaggedFileExt},
    tag::Accessor,
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use walkdir::WalkDir;

pub const EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "mp4", "aac", "alac"];

#[derive(Clone, Debug, Default)]
pub struct ScanProgress {
    pub discovered: usize,
    pub imported: usize,
    pub skipped: usize,
    pub current: String,
}

pub struct ScanResult {
    pub songs: Vec<Song>,
    pub seen: HashSet<PathBuf>,
    pub warnings: Vec<String>,
    pub can_prune: bool,
    pub cancelled: bool,
}

pub fn supported(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| EXTENSIONS.contains(&s.to_ascii_lowercase().as_str()))
}

pub fn scan(
    root: &Path,
    covers: &Path,
    cancel: &AtomicBool,
    progress: impl FnMut(ScanProgress),
) -> Result<ScanResult> {
    scan_cached(root, covers, cancel, &[], progress)
}

pub fn scan_cached(
    root: &Path,
    covers: &Path,
    cancel: &AtomicBool,
    existing: &[Song],
    mut progress: impl FnMut(ScanProgress),
) -> Result<ScanResult> {
    let existing: HashMap<_, _> = existing.iter().map(|s| (s.path.as_path(), s)).collect();
    if !root.is_dir() {
        bail!("Music folder is unavailable: {}", root.display());
    }
    let mut result = ScanResult {
        songs: Vec::new(),
        seen: HashSet::new(),
        warnings: Vec::new(),
        can_prune: true,
        cancelled: false,
    };
    let mut status = ScanProgress::default();
    for entry in WalkDir::new(root).follow_links(false) {
        if cancel.load(Ordering::Relaxed) {
            result.cancelled = true;
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                result.can_prune = false;
                if result.warnings.len() < 20 {
                    result.warnings.push(e.to_string());
                }
                continue;
            }
        };
        if !entry.file_type().is_file() || !supported(entry.path()) {
            continue;
        }
        let path = entry.path().to_path_buf();
        result.seen.insert(path.clone());
        status.discovered += 1;
        status.current = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let (size, modified, sidecar) = fingerprint(&path);
        if existing.get(path.as_path()).is_some_and(|s| {
            s.details.file_size == size
                && modified > 0
                && s.details.modified_ns == modified
                && s.details.sidecar_modified_ns == sidecar
                && s.cover.as_ref().is_none_or(|p| p.exists())
        }) {
            status.skipped += 1;
            if status.discovered % 20 == 0 {
                progress(status.clone());
            }
            continue;
        }
        match read_song(&path, covers) {
            Ok(song) => {
                result.songs.push(song);
                status.imported += 1;
            }
            Err(e) => {
                status.skipped += 1;
                if result.warnings.len() < 20 {
                    result.warnings.push(format!("{}: {e:#}", path.display()));
                }
            }
        }
        if status.discovered % 20 == 0 {
            progress(status.clone());
        }
    }
    progress(status);
    Ok(result)
}

pub fn read_song(path: &Path, covers: &Path) -> Result<Song> {
    let tagged = lofty::read_from_path(path).with_context(|| "Could not read audio metadata")?;
    let props = tagged.properties();
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let title = tag
        .and_then(|t| t.title())
        .map(|s| s.into_owned())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });
    let artist = tag
        .and_then(|t| t.artist())
        .map(|s| s.into_owned())
        .unwrap_or_else(|| "Unknown Artist".into());
    let album = tag
        .and_then(|t| t.album())
        .map(|s| s.into_owned())
        .unwrap_or_else(|| "Unknown Album".into());
    let album_artist = tag
        .and_then(|t| t.get_string(lofty::tag::ItemKey::AlbumArtist))
        .unwrap_or(&artist)
        .to_owned();
    let cover = tag
        .and_then(|t| t.pictures().first())
        .and_then(|pic| cache_cover(pic.data(), covers).ok());
    Ok(Song {
        path: path.to_path_buf(),
        title,
        artist,
        album,
        album_artist,
        duration: props.duration().as_secs_f64(),
        cover,
        genre: tag
            .and_then(|t| t.genre())
            .map(|s| s.into_owned())
            .unwrap_or_default(),
        year: tag
            .and_then(|t| {
                t.get_string(lofty::tag::ItemKey::RecordingDate)
                    .or_else(|| t.get_string(lofty::tag::ItemKey::Year))
            })
            .and_then(|s| s.get(..4))
            .and_then(|s| s.parse().ok()),
        track: tag.and_then(|t| t.track()),
        disc: tag.and_then(|t| t.disk()),
        details: {
            let (file_size, modified_ns, sidecar_modified_ns) = fingerprint(path);
            let embedded = tag
                .and_then(|t| {
                    t.get_string(lofty::tag::ItemKey::Lyrics)
                        .or_else(|| t.get_string(lofty::tag::ItemKey::UnsyncLyrics))
                })
                .unwrap_or_default();
            let sidecar = path.with_extension("lrc");
            let synced = tag.and_then(embedded_synced_lyrics);
            let lyrics = std::fs::metadata(&sidecar)
                .ok()
                .filter(|m| m.len() <= 2_000_000)
                .and_then(|_| std::fs::read_to_string(&sidecar).ok())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| synced.unwrap_or_else(|| embedded.to_owned()));
            let mut tags = std::collections::BTreeMap::new();
            if let Some(tag) = tag {
                for (name, key) in [
                    ("Composer", lofty::tag::ItemKey::Composer),
                    ("Conductor", lofty::tag::ItemKey::Conductor),
                    ("Lyricist", lofty::tag::ItemKey::Lyricist),
                    ("Comment", lofty::tag::ItemKey::Comment),
                    ("Language", lofty::tag::ItemKey::Language),
                    ("Mood", lofty::tag::ItemKey::Mood),
                    ("Key", lofty::tag::ItemKey::InitialKey),
                    ("BPM", lofty::tag::ItemKey::Bpm),
                    ("ReplayGain track", lofty::tag::ItemKey::ReplayGainTrackGain),
                    ("ReplayGain peak", lofty::tag::ItemKey::ReplayGainTrackPeak),
                    ("ReplayGain album", lofty::tag::ItemKey::ReplayGainAlbumGain),
                ] {
                    if let Some(value) = tag.get_string(key) {
                        tags.insert(name.into(), value.into());
                    }
                }
            }
            TrackDetails {
                file_size,
                modified_ns,
                sidecar_modified_ns,
                bitrate: props.audio_bitrate(),
                sample_rate: props.sample_rate(),
                channels: props.channels(),
                bit_depth: props.bit_depth(),
                format: path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_uppercase(),
                lyrics,
                tags,
            }
        },
        ..Song::default()
    })
}

fn cache_cover(bytes: &[u8], covers: &Path) -> Result<PathBuf> {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hash);
    let target = covers.join(format!("{:016x}.png", hash.finish()));
    if !target.exists() {
        std::fs::create_dir_all(covers)?;
        let image = image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?;
        image
            .thumbnail(512, 512)
            .save_with_format(&target, image::ImageFormat::Png)?;
    }
    Ok(target)
}

pub fn fingerprint(path: &Path) -> (u64, u64, u64) {
    fn modified(path: &Path) -> u64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_nanos().min(u64::MAX as u128) as u64)
    }
    (
        std::fs::metadata(path).map_or(0, |m| m.len()),
        modified(path),
        modified(&path.with_extension("lrc")),
    )
}

pub fn cleanup_covers(covers: &Path, songs: &[Song]) -> Result<usize> {
    let used: HashSet<_> = songs.iter().filter_map(|s| s.cover.as_ref()).collect();
    let mut removed = 0;
    for entry in std::fs::read_dir(covers)? {
        let entry = entry?;
        let path = entry.path();
        // Delete only cache files created by this scanner, never arbitrary user files.
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if entry.file_type()?.is_file()
            && path.extension().is_some_and(|s| s == "png")
            && stem.len() == 16
            && stem.chars().all(|c| c.is_ascii_hexdigit())
            && !used.contains(&path)
        {
            std::fs::remove_file(path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

fn embedded_synced_lyrics(tag: &lofty::tag::Tag) -> Option<String> {
    use lofty::{
        id3::v2::{Frame, FrameId, Id3v2Tag, SynchronizedTextFrame, TimestampFormat},
        tag::TagType,
    };
    if tag.tag_type() != TagType::Id3v2 {
        return None;
    }
    let original: Id3v2Tag = tag.clone().into();
    let Frame::Binary(frame) = original.get(&FrameId::Valid("SYLT".into()))? else {
        return None;
    };
    let lyrics = SynchronizedTextFrame::parse(&frame.data, frame.flags()).ok()?;
    if lyrics.timestamp_format != TimestampFormat::MS {
        return None;
    }
    Some(
        lyrics
            .content
            .iter()
            .map(|(ms, text)| {
                format!(
                    "[{:02}:{:02}.{:03}]{}",
                    ms / 60000,
                    (ms / 1000) % 60,
                    ms % 1000,
                    text
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

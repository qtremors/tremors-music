// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Disk operations run on the library worker, never on the UI thread.
use super::{LibraryCommand as Command, LibraryEvent as Event};
use crate::{library::Library, metadata, playlist_file, scanner, settings::AppPaths};
use anyhow::{Context, Result, ensure};
use std::time::Duration;

pub(super) fn execute(
    command: &Command,
    library: &mut Library,
    paths: &AppPaths,
) -> Option<Result<Event>> {
    if !matches!(
        command,
        Command::Backup(_)
            | Command::ReadBackup(_)
            | Command::RestoreBackup(_)
            | Command::ReadLyrics(_)
            | Command::SaveLyrics(_)
            | Command::ReadMetadata(_)
            | Command::SaveMetadata(..)
            | Command::LoadPicture(..)
            | Command::SetArtwork(..)
            | Command::ImportPlaylist(_)
            | Command::ExportPlaylist(..)
    ) {
        return None;
    }
    Some((|| match command {
        Command::Backup(path) => {
            crate::backup::write(library, path)?;
            Ok(Event::OperationFinished {
                message: "Curation backup saved.".into(),
                warnings: vec![],
            })
        }
        Command::ReadBackup(path) => Ok(Event::BackupLoaded(Box::new(crate::backup::read(path)?))),
        Command::RestoreBackup(restore) => Ok(Event::BackupRestored {
            warnings: crate::backup::restore(library, paths, restore)?,
            preferences: restore.preferences.then(|| restore.restored_preferences()),
        }),
        Command::ReadLyrics(id) => {
            let snapshot = library.full_snapshot()?;
            let song = snapshot
                .songs
                .iter()
                .find(|s| s.id == *id)
                .context("Track is no longer in the library")?;
            Ok(Event::LyricsLoaded(crate::lyrics::file::read(song)?))
        }
        Command::SaveLyrics(document) => {
            let snapshot = library.full_snapshot()?;
            let song = snapshot
                .songs
                .iter()
                .find(|s| s.id == document.song_id)
                .context("Track is no longer in the library")?;
            ensure!(
                ["lrc", "ttml", "txt"]
                    .into_iter()
                    .any(|ext| song.path.with_extension(ext) == document.path),
                "Track changed location; reopen the lyric editor."
            );
            crate::lyrics::file::save(document)?;
            let song = scanner::read_song(&song.path, &paths.covers)?;
            library.apply_scan(
                song.path.parent().context("Missing parent folder")?,
                std::slice::from_ref(&song),
                &Default::default(),
                false,
            )?;
            Ok(Event::LyricsLoaded(crate::lyrics::file::read(
                &crate::model::Song {
                    id: document.song_id,
                    ..song
                },
            )?))
        }
        Command::ReadMetadata(id) => {
            let snapshot = library.full_snapshot()?;
            let song = snapshot
                .songs
                .iter()
                .find(|s| s.id == *id)
                .context("Track is no longer in the library")?;
            Ok(Event::MetadataLoaded(Box::new(metadata::read(
                *id,
                &song.path,
                &paths.covers,
            )?)))
        }
        Command::SaveMetadata(document, release) => {
            ensure!(
                library
                    .full_snapshot()?
                    .songs
                    .iter()
                    .any(|s| s.id == document.song_id && s.path == document.path),
                "Track changed location or left the library. Reopen the editor."
            );
            if let Some(release) = release {
                release
                    .recv_timeout(Duration::from_secs(3))
                    .context("Playback could not release the file. Try saving again.")?;
            }
            metadata::save(document)?;
            let song = scanner::read_song(&document.path, &paths.covers)?;
            library.apply_scan(
                document.path.parent().context("Missing parent directory")?,
                &[song],
                &Default::default(),
                false,
            )?;
            Ok(Event::MetadataSaved(Box::new(metadata::read(
                document.song_id,
                &document.path,
                &paths.covers,
            )?)))
        }
        Command::LoadPicture(id, path) => Ok(Event::PictureLoaded(
            *id,
            metadata::load_artwork(path, &paths.covers)?,
        )),
        Command::SetArtwork(key, path) => {
            let cover = path
                .as_ref()
                .map(|p| metadata::load_artwork(p, &paths.covers))
                .transpose()?
                .and_then(|a| a.preview);
            library.set_artwork(key, cover.as_deref())?;
            Ok(Event::OperationFinished {
                message: "Artwork updated.".into(),
                warnings: vec![],
            })
        }
        Command::ImportPlaylist(path) => {
            let imported = playlist_file::read(path)?;
            let snapshot = library.full_snapshot()?;
            let mut warnings = imported.warnings;
            let mut songs = Vec::new();
            for path in imported.paths {
                if let Some(song) = snapshot.songs.iter().find(|s| s.path == path) {
                    songs.push(song.clone());
                    continue;
                }
                match scanner::read_song(&path, &paths.covers) {
                    Ok(song) => songs.push(song),
                    Err(error) => warnings.push(format!("{}: {error:#}", path.display())),
                }
            }
            ensure!(
                !songs.is_empty(),
                "No supported music files could be imported"
            );
            library.import_playlist(&imported.name, &songs)?;
            Ok(Event::OperationFinished {
                message: format!(
                    "Imported {} tracks into {}. {} warnings.",
                    songs.len(),
                    imported.name,
                    warnings.len()
                ),
                warnings,
            })
        }
        Command::ExportPlaylist(id, path, relative) => {
            let snapshot = library.full_snapshot()?;
            let playlist = snapshot
                .playlists
                .iter()
                .find(|p| p.id == *id)
                .context("Playlist no longer exists")?;
            let songs: Vec<_> = playlist
                .entries
                .iter()
                .filter_map(|e| snapshot.songs.iter().find(|s| s.id == e.song_id).cloned())
                .collect();
            playlist_file::write(path, &songs, *relative)?;
            Ok(Event::OperationFinished {
                message: format!("Exported {} tracks to {}", songs.len(), path.display()),
                warnings: vec![],
            })
        }
        _ => unreachable!(),
    })())
}

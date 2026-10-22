// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use std::{sync::atomic::Ordering, time::Duration};
use tremors_core::{
    service::{self, LibraryCommand, LibraryEvent},
    settings::AppPaths,
};

fn snapshot(handle: &service::LibraryHandle) -> std::sync::Arc<tremors_core::model::Snapshot> {
    loop {
        match handle
            .events
            .recv_timeout(Duration::from_secs(5))
            .expect("worker must respond")
        {
            LibraryEvent::Snapshot(snapshot) => return snapshot,
            LibraryEvent::Error(error) | LibraryEvent::MetadataError(_, error) => panic!("{error}"),
            _ => {}
        }
    }
}

#[test]
fn library_worker_runs_folder_scan_and_reports_real_results() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("music");
    std::fs::create_dir(&root).unwrap();
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/tone.flac"),
        root.join("song.flac"),
    )
    .unwrap();
    let handle = service::spawn(AppPaths::at(temp.path().join("data")).unwrap());
    assert!(snapshot(&handle).songs.is_empty());
    handle
        .commands
        .send(LibraryCommand::AddFolder(root))
        .unwrap();
    assert_eq!(snapshot(&handle).roots.len(), 1);
    handle.commands.send(LibraryCommand::Scan).unwrap();
    let mut started = false;
    let mut progress = false;
    let mut finished = false;
    loop {
        match handle.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            LibraryEvent::ScanStarted => started = true,
            LibraryEvent::ScanProgress(p) => {
                progress = true;
                assert_eq!(p.imported, 1);
            }
            LibraryEvent::ScanFinished { warnings, .. } => {
                finished = true;
                assert!(warnings.is_empty());
            }
            LibraryEvent::Snapshot(snapshot) => {
                assert_eq!(snapshot.songs.len(), 1);
                assert_eq!(snapshot.songs[0].title, "Fixture Tone");
                break;
            }
            LibraryEvent::Error(error) => panic!("{error}"),
            LibraryEvent::Warning(warning) => panic!("{warning}"),
            _ => {}
        }
    }
    assert!(started && progress && finished);
}

#[test]
fn early_cancellation_is_honored_and_next_scan_can_run() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("music");
    std::fs::create_dir(&root).unwrap();
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/tone.flac"),
        root.join("song.flac"),
    )
    .unwrap();
    let handle = service::spawn(AppPaths::at(temp.path().join("data")).unwrap());
    snapshot(&handle);
    handle
        .commands
        .send(LibraryCommand::AddFolder(root))
        .unwrap();
    snapshot(&handle);
    handle.cancel.store(true, Ordering::Relaxed);
    handle.commands.send(LibraryCommand::Scan).unwrap();
    assert!(snapshot(&handle).songs.is_empty());
    handle.cancel.store(false, Ordering::Relaxed);
    handle.commands.send(LibraryCommand::Scan).unwrap();
    assert_eq!(snapshot(&handle).songs.len(), 1);
}

#[test]
fn command_error_does_not_kill_the_library_worker() {
    let temp = tempfile::tempdir().unwrap();
    let handle = service::spawn(AppPaths::at(temp.path().join("data")).unwrap());
    snapshot(&handle);
    handle
        .commands
        .send(LibraryCommand::CreatePlaylist(" ".into()))
        .unwrap();
    assert!(matches!(
        handle.events.recv_timeout(Duration::from_secs(5)).unwrap(),
        LibraryEvent::Error(_)
    ));
    assert!(snapshot(&handle).playlists.is_empty());
    handle
        .commands
        .send(LibraryCommand::CreatePlaylist("Recovered".into()))
        .unwrap();
    assert_eq!(snapshot(&handle).playlists[0].name, "Recovered");
}

#[test]
fn metadata_and_playlist_operations_update_the_library_without_losing_curation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("music");
    std::fs::create_dir(&root).unwrap();
    let music = root.join("track.mp3");
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/tone.mp3"),
        &music,
    )
    .unwrap();
    let handle = service::spawn(AppPaths::at(temp.path().join("data")).unwrap());
    snapshot(&handle);
    handle
        .commands
        .send(LibraryCommand::AddFolderAndScan(root))
        .unwrap();
    snapshot(&handle);
    let first = snapshot(&handle);
    let id = first.songs[0].id;
    handle.commands.send(LibraryCommand::Favorite(id)).unwrap();
    snapshot(&handle);
    handle.commands.send(LibraryCommand::Played(id)).unwrap();
    snapshot(&handle);
    handle
        .commands
        .send(LibraryCommand::ReadMetadata(id))
        .unwrap();
    let mut doc = loop {
        match handle.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            LibraryEvent::MetadataLoaded(doc) => break doc,
            LibraryEvent::MetadataError(_, message) => panic!("{message}"),
            _ => {}
        }
    };
    snapshot(&handle);
    doc.fields
        .iter_mut()
        .find(|f| f.label == "Title")
        .unwrap()
        .value = "Saved title".into();
    let (release, rx) = std::sync::mpsc::channel();
    release.send(()).unwrap();
    handle
        .commands
        .send(LibraryCommand::SaveMetadata(doc, Some(rx)))
        .unwrap();
    let saved = snapshot(&handle);
    assert_eq!(saved.songs[0].id, id);
    assert_eq!(saved.songs[0].title, "Saved title");
    assert!(saved.songs[0].favorite);
    assert_eq!(saved.songs[0].play_count, 1);
    let playlist = temp.path().join("mix.m3u8");
    std::fs::write(
        &playlist,
        format!("#EXTM3U\n{}\n{}\n", music.display(), music.display()),
    )
    .unwrap();
    handle
        .commands
        .send(LibraryCommand::ImportPlaylist(playlist))
        .unwrap();
    let imported = snapshot(&handle);
    assert_eq!(imported.songs.len(), 1);
    assert_eq!(imported.playlists[0].entries.len(), 2);
    let output = temp.path().join("export.m3u8");
    handle
        .commands
        .send(LibraryCommand::ExportPlaylist(
            imported.playlists[0].id,
            output.clone(),
            true,
        ))
        .unwrap();
    snapshot(&handle);
    assert_eq!(
        tremors_core::playlist_file::read(&output).unwrap().paths,
        vec![music.clone(), music]
    );
    let image = temp.path().join("custom.png");
    image::RgbImage::from_pixel(16, 16, image::Rgb([20, 40, 60]))
        .save(&image)
        .unwrap();
    let key = format!("playlist:{}", imported.playlists[0].id);
    handle
        .commands
        .send(LibraryCommand::SetArtwork(key.clone(), Some(image)))
        .unwrap();
    let customized = snapshot(&handle);
    let cached = customized.artwork[&key].clone();
    handle.commands.send(LibraryCommand::CleanArtwork).unwrap();
    snapshot(&handle);
    assert!(cached.is_file(), "Cleanup must preserve custom artwork");
    handle
        .commands
        .send(LibraryCommand::DeletePlaylist(imported.playlists[0].id))
        .unwrap();
    assert!(!snapshot(&handle).artwork.contains_key(&key));
}

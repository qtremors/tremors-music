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
            LibraryEvent::Error(error) => panic!("{error}"),
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

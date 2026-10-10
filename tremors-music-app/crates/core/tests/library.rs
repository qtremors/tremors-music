// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use std::{
    collections::HashSet,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use tremors_core::{
    library::Library,
    model::Song,
    scanner,
    settings::{Preferences, RepeatMode},
};

fn track(root: &Path, name: &str) -> Song {
    Song {
        path: root.join(name),
        title: name.into(),
        artist: "Artist".into(),
        album: "Album".into(),
        album_artist: "Artist".into(),
        duration: 1.,
        ..Default::default()
    }
}

fn wav(path: &Path) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for _ in 0..8000 {
        writer.write_sample(100_i16).unwrap();
    }
    writer.finalize().unwrap();
}

#[test]
fn rescanning_preserves_identity_favorites_and_play_counts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let db = root.join("library.sqlite");
    let mut library = Library::open(&db).unwrap();
    let mut song = track(root, "one.mp3");
    let seen = HashSet::from([song.path.clone()]);
    library
        .apply_scan(root, &[song.clone()], &seen, true)
        .unwrap();
    let original = library.snapshot().unwrap().songs.remove(0);
    library.favorite(original.id).unwrap();
    library.played(original.id).unwrap();
    song.title = "Updated tag".into();
    library.apply_scan(root, &[song], &seen, true).unwrap();
    let current = library.snapshot().unwrap().songs.remove(0);
    assert_eq!(current.id, original.id);
    assert_eq!(current.added_at, original.added_at);
    assert!(current.favorite);
    assert_eq!(current.play_count, 1);
    assert_eq!(current.title, "Updated tag");
    drop(library);
    assert_eq!(
        Library::open(&db).unwrap().snapshot().unwrap().songs[0],
        current
    );
}

#[test]
fn duplicate_playlist_entries_can_be_reordered_and_removed_individually() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut library = Library::open(&root.join("library.sqlite")).unwrap();
    let tracks = vec![track(root, "a.mp3"), track(root, "b.mp3")];
    library
        .apply_scan(
            root,
            &tracks,
            &tracks.iter().map(|s| s.path.clone()).collect(),
            true,
        )
        .unwrap();
    let songs = library.snapshot().unwrap().songs;
    let playlist = library.create_playlist("  Test  ").unwrap();
    for id in [songs[0].id, songs[1].id, songs[0].id] {
        library.add_to_playlist(playlist, id).unwrap();
    }
    let entries = library.snapshot().unwrap().playlists[0].entries.clone();
    library.move_entry(playlist, entries[2].id, -1).unwrap();
    let changed = library.snapshot().unwrap().playlists[0].entries.clone();
    assert_eq!(
        changed.iter().map(|e| e.id).collect::<Vec<_>>(),
        vec![entries[0].id, entries[2].id, entries[1].id]
    );
    library.remove_entry(entries[0].id).unwrap();
    assert_eq!(library.snapshot().unwrap().playlists[0].entries.len(), 2);
    library.rename_playlist(playlist, "Renamed").unwrap();
    assert_eq!(library.snapshot().unwrap().playlists[0].name, "Renamed");
    assert!(library.create_playlist(" ").is_err());
    assert!(library.add_to_playlist(playlist, 9999).is_err());
    library.delete_playlist(playlist).unwrap();
    assert!(library.snapshot().unwrap().playlists.is_empty());
}

#[test]
fn prune_only_removes_missing_tracks_inside_the_scanned_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("music");
    let other = temp.path().join("other");
    std::fs::create_dir_all(&root).unwrap();
    let mut library = Library::open(&temp.path().join("library.sqlite")).unwrap();
    let tracks = vec![
        track(&root, "a.mp3"),
        track(&root, "b.mp3"),
        track(&other, "c.mp3"),
    ];
    library
        .apply_scan(
            temp.path(),
            &tracks,
            &tracks.iter().map(|s| s.path.clone()).collect(),
            false,
        )
        .unwrap();
    let playlist = library.create_playlist("Pruning").unwrap();
    let songs = library.snapshot().unwrap().songs;
    library.add_to_playlist(playlist, songs[1].id).unwrap();
    library
        .apply_scan(&root, &[], &HashSet::from([tracks[0].path.clone()]), true)
        .unwrap();
    let snapshot = library.snapshot().unwrap();
    assert_eq!(snapshot.songs.len(), 2);
    assert!(snapshot.songs.iter().any(|s| s.path == tracks[2].path));
    assert!(snapshot.playlists[0].entries.is_empty());
}

#[test]
fn failed_or_partial_traversal_can_disable_pruning() {
    let temp = tempfile::tempdir().unwrap();
    let mut library = Library::open(&temp.path().join("library.sqlite")).unwrap();
    let song = track(temp.path(), "a.mp3");
    library
        .apply_scan(temp.path(), &[song], &HashSet::new(), false)
        .unwrap();
    library
        .apply_scan(temp.path(), &[], &HashSet::new(), false)
        .unwrap();
    assert_eq!(library.snapshot().unwrap().songs.len(), 1);
}

#[test]
fn removing_overlapping_roots_preserves_tracks_covered_by_remaining_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(temp.path()).unwrap();
    let nested = root.join("album");
    std::fs::create_dir(&nested).unwrap();
    let mut library = Library::open(&root.join("library.sqlite")).unwrap();
    library.add_root(&root).unwrap();
    library.add_root(&nested).unwrap();
    let tracks = vec![track(&root, "a.mp3"), track(&nested, "b.mp3")];
    library
        .apply_scan(
            &root,
            &tracks,
            &tracks.iter().map(|s| s.path.clone()).collect(),
            true,
        )
        .unwrap();
    library.remove_root(&root).unwrap();
    assert_eq!(library.snapshot().unwrap().songs[0].path, tracks[1].path);
    library.remove_root(&nested).unwrap();
    assert!(library.snapshot().unwrap().songs.is_empty());
}

#[test]
fn preferences_survive_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("db.sqlite");
    let mut library = Library::open(&path).unwrap();
    library
        .save_preferences(&Preferences {
            volume: 0.25,
            shuffle: true,
            repeat: RepeatMode::One,
            ..Default::default()
        })
        .unwrap();
    drop(library);
    let prefs = Library::open(&path)
        .unwrap()
        .snapshot()
        .unwrap()
        .preferences;
    assert_eq!(prefs.volume, 0.25);
    assert!(prefs.shuffle);
    assert_eq!(prefs.repeat, RepeatMode::One);
}

#[test]
fn scanner_reads_real_audio_and_keeps_corrupt_paths_seen() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("music");
    std::fs::create_dir(&root).unwrap();
    wav(&root.join("Track.WAV"));
    std::fs::write(root.join("bad.mp3"), "not audio").unwrap();
    std::fs::write(root.join("ignored.wma"), "not supported").unwrap();
    let cancel = AtomicBool::new(false);
    let mut last = None;
    let scan = scanner::scan(&root, &temp.path().join("covers"), &cancel, |p| {
        last = Some(p)
    })
    .unwrap();
    assert_eq!(scan.songs.len(), 1);
    assert_eq!(scan.songs[0].title, "Track");
    assert!((scan.songs[0].duration - 1.).abs() < 0.01);
    assert_eq!(scan.seen.len(), 2);
    assert_eq!(scan.warnings.len(), 1);
    assert_eq!(last.unwrap().skipped, 1);
    assert!(scan.can_prune);
    assert!(!scan.cancelled);
}

#[test]
fn cancellation_does_not_return_a_completed_scan() {
    let temp = tempfile::tempdir().unwrap();
    let cancel = AtomicBool::new(true);
    wav(&temp.path().join("track.wav"));
    let scan = scanner::scan(temp.path(), &temp.path().join("covers"), &cancel, |_| {}).unwrap();
    assert!(scan.cancelled);
    assert!(scan.songs.is_empty());
    cancel.store(false, Ordering::Relaxed);
    assert!(scanner::scan(&temp.path().join("missing"), temp.path(), &cancel, |_| {}).is_err());
}

#[test]
fn wav_decoding_and_seeking_use_the_actual_playback_decoder() {
    use rodio::Source;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("audio.wav");
    wav(&path);
    let mut source = rodio::Decoder::try_from(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(source.total_duration().unwrap().as_secs(), 1);
    assert!(source.next().unwrap() > 0.);
    source
        .try_seek(std::time::Duration::from_millis(500))
        .unwrap();
    assert!(source.count() > 0);
}

#[test]
fn unreadable_tracks_seen_during_rescan_keep_existing_user_data() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("music");
    std::fs::create_dir(&root).unwrap();
    let path = root.join("track.wav");
    wav(&path);
    let mut library = Library::open(&temp.path().join("library.sqlite")).unwrap();
    let cancel = AtomicBool::new(false);
    let first = scanner::scan(&root, &temp.path().join("covers"), &cancel, |_| {}).unwrap();
    library
        .apply_scan(&root, &first.songs, &first.seen, first.can_prune)
        .unwrap();
    let id = library.snapshot().unwrap().songs[0].id;
    library.favorite(id).unwrap();
    library.played(id).unwrap();
    std::fs::write(path, "temporarily unreadable audio").unwrap();
    let second = scanner::scan(&root, &temp.path().join("covers"), &cancel, |_| {}).unwrap();
    assert_eq!(second.warnings.len(), 1);
    assert!(second.songs.is_empty());
    library
        .apply_scan(&root, &second.songs, &second.seen, second.can_prune)
        .unwrap();
    let song = library.snapshot().unwrap().songs.remove(0);
    assert_eq!(song.id, id);
    assert!(song.favorite);
    assert_eq!(song.play_count, 1);
}

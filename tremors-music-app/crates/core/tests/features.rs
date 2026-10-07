// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use std::{collections::HashSet, path::Path, sync::atomic::AtomicBool, time::Duration};
use tremors_core::{
    browsing::{search_rank, sort_songs},
    library::Library,
    lyrics::{active_line, parse_lrc},
    model::{Song, TrackDetails},
    playback::track_gain,
    queue::{Queue, QueueState},
    scanner,
    settings::{Preferences, SongSort},
};

fn song(id: i64) -> Song {
    Song {
        id,
        title: format!("Song {id}"),
        duration: 60.,
        ..Default::default()
    }
}

#[test]
fn play_next_and_reordering_preserve_current_duplicate_identity() {
    let mut q = Queue::default();
    q.replace(vec![song(1), song(2), song(1)], 2);
    q.play_next(song(3));
    assert_eq!(q.current_index(), Some(2));
    assert_eq!(q.next(false).unwrap().id, 3);
    q.move_to(3, 0);
    assert_eq!(q.current_index(), Some(0));
    assert_eq!(q.current().unwrap().id, 3);
    q.set_shuffle(true);
    q.play_next(song(4));
    assert_eq!(q.next(false).unwrap().id, 4);
    q.set_shuffle(false);
    assert_eq!(q.current().unwrap().id, 4);
}

#[test]
fn saved_shuffle_restores_exact_order_position_and_duplicates() {
    let mut q = Queue::default();
    q.replace(vec![song(1), song(2), song(1), song(3)], 2);
    q.set_shuffle(true);
    q.move_to(0, 2);
    let saved = q.state(22.5);
    let serialized = serde_json::to_string(&saved).unwrap();
    let saved: QueueState = serde_json::from_str(&serialized).unwrap();
    let mut restored = Queue::default();
    restored.shuffle = true;
    assert_eq!(restored.restore(&saved, &[song(1), song(2), song(3)]), 22.5);
    assert_eq!(restored.current_index(), q.current_index());
    assert_eq!(restored.ordered(), q.ordered());
    restored.set_shuffle(false);
    q.set_shuffle(false);
    assert_eq!(restored.ordered(), q.ordered());
}

#[test]
fn missing_and_corrupt_session_entries_cannot_select_the_wrong_song() {
    let saved = QueueState {
        song_ids: vec![1, 2, 1],
        order: vec![0, 1, 2],
        cursor: Some(2),
        position: 15.,
    };
    let mut q = Queue::default();
    assert_eq!(q.restore(&saved, &[song(1)]), 15.);
    assert_eq!(q.current_index(), Some(1));
    let missing = QueueState {
        cursor: Some(1),
        ..saved.clone()
    };
    assert_eq!(q.restore(&missing, &[song(1)]), 0.);
    assert!(q.current().is_none());
    let corrupt = QueueState {
        order: vec![900, 900],
        position: f64::NAN,
        ..saved
    };
    assert_eq!(q.restore(&corrupt, &[song(1), song(2)]), 0.);
    assert_eq!(q.len(), 3);
}

#[test]
fn lrc_offsets_multiple_timestamps_and_active_lines_are_correct() {
    let lines = parse_lrc(
        "[ar:Example]\n[offset:-500]\n[00:01.00][00:03.500]Hello\n[00:02.25]World\n[00:99]Invalid\n[bad]Ignored",
    );
    assert_eq!(
        lines.iter().map(|l| l.seconds).collect::<Vec<_>>(),
        vec![0.5, 1.75, 3.]
    );
    assert_eq!(active_line(&lines, 0.), None);
    assert_eq!(active_line(&lines, 1.75), Some(1));
    assert_eq!(lines[2].text, "Hello");
    assert!(parse_lrc("just plain lyrics").is_empty());
}

#[test]
fn replaygain_limits_peaks_and_rejects_invalid_tags() {
    let mut details = TrackDetails::default();
    details
        .tags
        .insert("ReplayGain track".into(), "6.0 dB".into());
    assert_eq!(track_gain(&details, false), 1.);
    assert!((track_gain(&details, true) - 1.9952624).abs() < 0.0001);
    details.tags.insert("ReplayGain peak".into(), "0.8".into());
    assert_eq!(track_gain(&details, true), 1.25);
    details.tags.insert("ReplayGain track".into(), "NaN".into());
    assert_eq!(track_gain(&details, true), 1.);
}

#[test]
fn database_persists_session_metadata_and_new_preferences() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("library.sqlite3");
    let root = temp.path().join("music");
    std::fs::create_dir(&root).unwrap();
    let mut s = song(0);
    s.path = root.join("song.flac");
    s.details.lyrics = "[00:01]Saved".into();
    s.details.file_size = 42;
    let mut lib = Library::open(&path).unwrap();
    lib.add_root(&root).unwrap();
    lib.apply_scan(&root, &[s.clone()], &HashSet::from([s.path]), true)
        .unwrap();
    let id = lib.snapshot().unwrap().songs[0].id;
    let session = QueueState {
        song_ids: vec![id, id],
        order: vec![1, 0],
        cursor: Some(1),
        position: 12.,
    };
    lib.save_session(&session).unwrap();
    lib.save_preferences(&Preferences {
        light_theme: true,
        accent: "Teal".into(),
        sort: SongSort::Year,
        descending: true,
        ..Default::default()
    })
    .unwrap();
    drop(lib);
    let mut lib = Library::open(&path).unwrap();
    let snap = lib.snapshot().unwrap();
    assert_eq!(snap.songs[0].details.lyrics, "[00:01]Saved");
    assert_eq!(snap.session.song_ids, vec![id, id]);
    assert!(snap.preferences.light_theme);
    lib.reset(false).unwrap();
    let snap = lib.snapshot().unwrap();
    assert!(snap.songs.is_empty());
    assert_eq!(snap.roots.len(), 1);
    assert!(snap.session.song_ids.is_empty());
    assert!(snap.preferences.light_theme);
    lib.reset(true).unwrap();
    assert!(lib.snapshot().unwrap().roots.is_empty());
}

#[test]
fn rescan_skips_unchanged_tracks_but_rereads_changed_sidecar_lyrics() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("music");
    std::fs::create_dir(&root).unwrap();
    let path = root.join("song.flac");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.flac"),
        &path,
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    let first = scanner::scan(&root, temp.path(), &cancel, |_| {}).unwrap();
    assert_eq!(first.songs.len(), 1);
    let mut skipped = 0;
    let second = scanner::scan_cached(&root, temp.path(), &cancel, &first.songs, |p| {
        skipped = p.skipped
    })
    .unwrap();
    assert!(second.songs.is_empty());
    assert_eq!(skipped, 1);
    assert!(second.seen.contains(&path));
    std::fs::write(path.with_extension("lrc"), "[00:01]New lyric").unwrap();
    let third = scanner::scan_cached(&root, temp.path(), &cancel, &first.songs, |_| {}).unwrap();
    assert_eq!(third.songs[0].details.lyrics, "[00:01]New lyric");
}

#[test]
fn sort_and_search_prioritize_exact_titles_and_support_numeric_fields() {
    let mut a = song(1);
    a.title = "Blue".into();
    a.year = Some(2020);
    let mut b = song(2);
    b.title = "Blue sky".into();
    b.year = Some(2024);
    assert!(search_rank(&a, "Blue") < search_rank(&b, "Blue"));
    let mut songs = vec![a, b];
    sort_songs(&mut songs, SongSort::Year, true);
    assert_eq!(songs[0].id, 2);
}

#[test]
fn folder_watcher_imports_new_music_without_a_manual_scan() {
    use tremors_core::{
        service::{self, LibraryCommand, LibraryEvent},
        settings::AppPaths,
    };
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("music");
    std::fs::create_dir(&root).unwrap();
    let handle = service::spawn(AppPaths::at(temp.path().join("data")).unwrap());
    let receive = || loop {
        if let LibraryEvent::Snapshot(s) =
            handle.events.recv_timeout(Duration::from_secs(12)).unwrap()
        {
            break s;
        }
    };
    receive();
    handle
        .commands
        .send(LibraryCommand::AddFolder(root.clone()))
        .unwrap();
    receive();
    // A round trip ensures watches were registered after AddFolder.
    handle.commands.send(LibraryCommand::Refresh).unwrap();
    receive();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.flac"),
        root.join("new.flac"),
    )
    .unwrap();
    let snapshot = receive();
    assert_eq!(snapshot.songs.len(), 1);
    assert_eq!(snapshot.songs[0].title, "Fixture Tone");
}

#[test]
fn embedded_id3_synchronized_lyrics_keep_millisecond_timestamps() {
    use lofty::{
        TextEncoding,
        id3::v2::{
            BinaryFrame, Frame, FrameId, Id3v2Tag, SyncTextContentType, SynchronizedTextFrame,
            TimestampFormat,
        },
        tag::{TagExt, TagType},
    };
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("lyrics.mp3");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3"),
        &path,
    )
    .unwrap();
    let text = SynchronizedTextFrame::new(
        TextEncoding::UTF8,
        *b"eng",
        TimestampFormat::MS,
        SyncTextContentType::Lyrics,
        None,
        vec![(1250, "First".into()), (2700, "Second".into())],
    );
    let mut tag = Id3v2Tag::default();
    tag.insert(Frame::Binary(BinaryFrame::new(
        FrameId::Valid("SYLT".into()),
        text.as_bytes(lofty::config::WriteOptions::default())
            .unwrap(),
    )));
    let tag: lofty::tag::Tag = tag.into();
    assert_eq!(tag.tag_type(), TagType::Id3v2);
    tag.save_to_path(&path, lofty::config::WriteOptions::default())
        .unwrap();
    let song = scanner::read_song(&path, temp.path()).unwrap();
    let lines = parse_lrc(&song.details.lyrics);
    assert_eq!(lines[0].seconds, 1.25);
    assert_eq!(lines[1].text, "Second");
}

#[test]
fn relocation_preserves_playlist_identity_and_drops_unavailable_old_paths() {
    let temp = tempfile::tempdir().unwrap();
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    std::fs::create_dir(&old).unwrap();
    std::fs::create_dir(&new).unwrap();
    std::fs::write(new.join("song.flac"), b"placeholder").unwrap();
    let mut a = song(0);
    a.path = old.join("song.flac");
    let mut b = song(0);
    b.path = old.join("missing.flac");
    let mut lib = Library::open(&temp.path().join("db.sqlite")).unwrap();
    lib.add_root(&old).unwrap();
    let seen = HashSet::from([a.path.clone(), b.path.clone()]);
    lib.apply_scan(&old, &[a, b], &seen, true).unwrap();
    let id = lib
        .snapshot()
        .unwrap()
        .songs
        .iter()
        .find(|s| s.path.ends_with("song.flac"))
        .unwrap()
        .id;
    lib.favorite(id).unwrap();
    let playlist = lib.create_playlist("Saved").unwrap();
    lib.add_to_playlist(playlist, id).unwrap();
    lib.replace_root(&old, &new).unwrap();
    let snapshot = lib.snapshot().unwrap();
    assert_eq!(snapshot.songs.len(), 1);
    assert_eq!(snapshot.songs[0].id, id);
    assert!(snapshot.songs[0].favorite);
    assert_eq!(snapshot.songs[0].path, new.join("song.flac"));
    assert_eq!(snapshot.playlists[0].entries[0].song_id, id);
}

#[test]
fn cover_cleanup_only_removes_unreferenced_generated_files() {
    let temp = tempfile::tempdir().unwrap();
    let referenced = temp.path().join("0123456789abcdef.png");
    let stale = temp.path().join("fedcba9876543210.png");
    let user = temp.path().join("my-cover.png");
    for path in [&referenced, &stale, &user] {
        std::fs::write(path, b"data").unwrap();
    }
    let mut s = song(1);
    s.cover = Some(referenced.clone());
    assert_eq!(scanner::cleanup_covers(temp.path(), &[s]).unwrap(), 1);
    assert!(referenced.exists() && user.exists());
    assert!(!stale.exists());
}

#[test]
fn library_refresh_updates_metadata_and_preserves_current_duplicate() {
    let mut queue = Queue::default();
    queue.replace(vec![song(1), song(2), song(1), song(3)], 2);
    let mut updated = song(1);
    updated.title = "Updated title".into();
    assert!(!queue.reconcile(&[updated.clone()]));
    assert_eq!(queue.current_index(), Some(1));
    assert_eq!(queue.current().unwrap().title, "Updated title");
    assert_eq!(queue.len(), 2);
    assert!(queue.reconcile(&[]));
    assert!(queue.current().is_none());
}

#[test]
fn adding_an_invalid_folder_cannot_start_an_unrelated_scan() {
    use tremors_core::{
        service::{self, LibraryCommand, LibraryEvent},
        settings::AppPaths,
    };
    let temp = tempfile::tempdir().unwrap();
    let handle = service::spawn(AppPaths::at(temp.path().join("data")).unwrap());
    handle.events.recv_timeout(Duration::from_secs(5)).unwrap();
    handle
        .commands
        .send(LibraryCommand::AddFolderAndScan(
            temp.path().join("missing"),
        ))
        .unwrap();
    assert!(matches!(
        handle.events.recv_timeout(Duration::from_secs(5)).unwrap(),
        LibraryEvent::Error(_)
    ));
    handle.commands.send(LibraryCommand::Refresh).unwrap();
    assert!(matches!(
        handle.events.recv_timeout(Duration::from_secs(5)).unwrap(),
        LibraryEvent::Snapshot(_)
    ));
}

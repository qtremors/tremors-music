// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use std::{
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
use tremors_core::{
    browsing::{BrowsePage, build_rows, visible_songs},
    library::Library,
    model::Song,
    playlist_file,
    queue::Queue,
    scanner,
    settings::Preferences,
    sleep_timer::{SleepTimer, TimerState},
};
fn copy_tone(root: &Path, name: &str) -> Song {
    let path = root.join(name);
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.flac"),
        &path,
    )
    .unwrap();
    scanner::read_song(&path, &root.join("covers")).unwrap()
}
#[test]
fn portable_playlists_preserve_unicode_relative_paths_and_duplicates() {
    let temp = tempfile::tempdir().unwrap();
    let song = copy_tone(temp.path(), "世界.flac");
    let path = temp.path().join("mix.m3u8");
    playlist_file::write(&path, &[song.clone(), song.clone()], true).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(!content.contains(&temp.path().display().to_string()));
    std::fs::write(
        &path,
        format!("\u{feff}{content}\nmissing.flac\nhttps://example.invalid/audio\n"),
    )
    .unwrap();
    let playlist = playlist_file::read(&path).unwrap();
    assert_eq!(playlist.paths, vec![song.path.clone(), song.path]);
    assert_eq!(playlist.warnings.len(), 2);
}
#[test]
fn failed_export_keeps_existing_playlist_and_file_urls_import() {
    let temp = tempfile::tempdir().unwrap();
    let song = copy_tone(temp.path(), "space name.flac");
    let path = temp.path().join("mix.m3u8");
    std::fs::write(
        &path,
        format!(
            "#EXTM3U\n{}\n",
            url::Url::from_file_path(&song.path).unwrap()
        ),
    )
    .unwrap();
    assert_eq!(
        playlist_file::read(&path).unwrap().paths,
        vec![song.path.clone()]
    );
    let audio = std::fs::read(&song.path).unwrap();
    assert!(playlist_file::write(&song.path, std::slice::from_ref(&song), true).is_err());
    assert_eq!(std::fs::read(&song.path).unwrap(), audio);
    let before = std::fs::read(&path).unwrap();
    assert!(
        playlist_file::write(
            &path,
            &[Song {
                path: temp.path().join("missing.flac"),
                ..Default::default()
            }],
            true
        )
        .is_err()
    );
    assert_eq!(std::fs::read(path).unwrap(), before);
}
#[test]
fn excluded_tracks_keep_library_identity_history_and_playlist_order() {
    let temp = tempfile::tempdir().unwrap();
    let song = copy_tone(temp.path(), "track.flac");
    let mut library = Library::open(&temp.path().join("library.sqlite")).unwrap();
    library.add_root(temp.path()).unwrap();
    let playlist = library
        .import_playlist("Mix", &[song.clone(), song])
        .unwrap();
    let snapshot = library.snapshot().unwrap();
    let id = snapshot.songs[0].id;
    library.favorite(id).unwrap();
    library.played(id).unwrap();
    let preferences = Preferences {
        excluded_folders: vec![temp.path().to_path_buf()],
        ..Default::default()
    };
    library.save_preferences(&preferences).unwrap();
    assert!(library.snapshot().unwrap().songs.is_empty());
    let full = library.full_snapshot().unwrap();
    let scan = scanner::scan_cached_filtered(
        temp.path(),
        &temp.path().join("covers"),
        &AtomicBool::new(false),
        &full.songs,
        &preferences,
        |_| {},
    )
    .unwrap();
    library
        .apply_scan(temp.path(), &scan.songs, &scan.seen, scan.can_prune)
        .unwrap();
    library.save_preferences(&Preferences::default()).unwrap();
    let snapshot = library.snapshot().unwrap();
    assert_eq!(snapshot.songs[0].id, id);
    assert!(snapshot.songs[0].favorite);
    assert_eq!(snapshot.songs[0].play_count, 1);
    assert_eq!(snapshot.playlists[0].id, playlist);
    assert_eq!(snapshot.playlists[0].entries.len(), 2);
}
#[test]
fn recently_played_is_sorted_by_last_play_without_unplayed_tracks() {
    let snapshot = tremors_core::model::Snapshot {
        songs: vec![
            Song {
                id: 1,
                last_played: Some(100),
                ..Default::default()
            },
            Song {
                id: 2,
                last_played: Some(200),
                ..Default::default()
            },
            Song {
                id: 3,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let rows = build_rows(
        &snapshot,
        &BrowsePage::RecentlyPlayed,
        "",
        &Queue::default(),
        &Preferences::default(),
    );
    assert_eq!(
        visible_songs(&rows)
            .iter()
            .map(|s| s.id)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
}
#[test]
fn sleep_timer_fades_and_can_finish_a_track_but_expires_while_paused() {
    let now = Instant::now();
    let timer = SleepTimer::new(now, Duration::from_secs(30), false, Duration::from_secs(10));
    assert!(
        matches!(timer.state(now+Duration::from_secs(25),true,None),TimerState::Running{gain,..} if (gain-0.5).abs()<0.001)
    );
    assert_eq!(
        timer.state(now + Duration::from_secs(30), true, None),
        TimerState::Expired
    );
    let timer = SleepTimer::new(now, Duration::from_secs(30), true, Duration::from_secs(10));
    assert_eq!(
        timer.state(
            now + Duration::from_secs(31),
            true,
            Some(Duration::from_secs(5))
        ),
        TimerState::Finishing { gain: 0.5 }
    );
    assert_eq!(
        timer.state(now + Duration::from_secs(31), false, None),
        TimerState::Expired
    );
    assert!(timer.stop_at_track_end(now + Duration::from_secs(30)));
}
#[test]
fn folder_art_changes_invalidate_scanner_cache() {
    let temp = tempfile::tempdir().unwrap();
    let song = copy_tone(temp.path(), "track.flac");
    assert!(song.cover.is_none());
    let cover = temp.path().join("folder.jpg");
    image::RgbImage::from_pixel(16, 16, image::Rgb([10, 20, 30]))
        .save(&cover)
        .unwrap();
    let scan = scanner::scan_cached(
        temp.path(),
        &temp.path().join("covers"),
        &AtomicBool::new(false),
        &[song],
        |_| {},
    )
    .unwrap();
    assert!(scan.songs[0].cover.as_ref().unwrap().exists());
    std::fs::remove_file(cover).unwrap();
    let scan = scanner::scan_cached(
        temp.path(),
        &temp.path().join("covers"),
        &AtomicBool::new(false),
        &scan.songs,
        |_| {},
    )
    .unwrap();
    assert!(scan.songs[0].cover.is_none());
}

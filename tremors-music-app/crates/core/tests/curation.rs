// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors
use tremors_core::{
    backup::{self, Restore},
    browsing::{self, BrowsePage, BrowseRow},
    library::Library,
    model::Song,
    queue::Queue,
    settings::AppPaths,
};
fn add(library: &mut Library, paths: &[std::path::PathBuf]) {
    let tracks: Vec<_> = paths
        .iter()
        .map(|path| Song {
            path: path.clone(),
            title: path.file_name().unwrap().to_string_lossy().into_owned(),
            duration: 30.,
            ..Default::default()
        })
        .collect();
    library
        .apply_scan(
            paths[0].parent().unwrap(),
            &tracks,
            &Default::default(),
            false,
        )
        .unwrap();
}
#[test]
fn backup_remaps_paths_preserves_duplicate_order_and_merges_curation() {
    let temp = tempfile::tempdir().unwrap();
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    let mut source = Library::open(&temp.path().join("source.sqlite")).unwrap();
    add(
        &mut source,
        &[
            old.join("a.mp3"),
            old.join("b.mp3"),
            old.join("missing.mp3"),
        ],
    );
    let tracks = source.full_snapshot().unwrap().songs;
    let a = tracks.iter().find(|s| s.title == "a.mp3").unwrap().id;
    let b = tracks.iter().find(|s| s.title == "b.mp3").unwrap().id;
    source.favorite(a).unwrap();
    source.played(a).unwrap();
    let pl = source.create_playlist("Mix").unwrap();
    source.add_many_to_playlist(pl, &[b, a, b]).unwrap();
    let file = temp.path().join("curation.tremors-backup");
    backup::write(&source, &file).unwrap();
    let document = backup::read(&file).unwrap();
    let paths = AppPaths::at(temp.path().join("target")).unwrap();
    let mut target = Library::open(&paths.database).unwrap();
    add(&mut target, &[new.join("b.mp3"), new.join("a.mp3")]);
    target.create_playlist("Existing").unwrap();
    let request = Restore {
        backup: document,
        remap: Some((old, new)),
        preferences: false,
    };
    let warnings = backup::restore(&mut target, &paths, &request).unwrap();
    assert_eq!(warnings.len(), 1);
    let result = target.full_snapshot().unwrap();
    assert!(result.playlists.iter().any(|p| p.name == "Existing"));
    let mix = result.playlists.iter().find(|p| p.name == "Mix").unwrap();
    let names: Vec<_> = mix
        .entries
        .iter()
        .map(|e| {
            result
                .songs
                .iter()
                .find(|s| s.id == e.song_id)
                .unwrap()
                .title
                .as_str()
        })
        .collect();
    assert_eq!(names, ["b.mp3", "a.mp3", "b.mp3"]);
    let a = result.songs.iter().find(|s| s.title == "a.mp3").unwrap();
    assert!(a.favorite);
    assert_eq!(a.play_count, 1);
    backup::restore(&mut target, &paths, &request).unwrap();
    assert_eq!(
        target
            .full_snapshot()
            .unwrap()
            .songs
            .iter()
            .find(|s| s.title == "a.mp3")
            .unwrap()
            .play_count,
        1
    );
}
#[test]
fn invalid_archive_and_zero_matches_leave_library_intact() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::at(temp.path().join("target")).unwrap();
    let mut library = Library::open(&paths.database).unwrap();
    add(&mut library, &[temp.path().join("a.mp3")]);
    library.create_playlist("Keep").unwrap();
    let file = temp.path().join("curation.tremors-backup");
    backup::write(&library, &file).unwrap();
    let mut doc = backup::read(&file).unwrap();
    doc.version = 99;
    std::fs::write(&file, serde_json::to_vec(&doc).unwrap()).unwrap();
    assert!(backup::read(&file).is_err());
    doc.version = 1;
    doc.songs[0].path = temp.path().join("absent.mp3");
    assert!(
        backup::restore(
            &mut library,
            &paths,
            &Restore {
                backup: doc,
                remap: None,
                preferences: false
            }
        )
        .is_err()
    );
    assert_eq!(library.full_snapshot().unwrap().playlists.len(), 1);
    let audio = temp.path().join("keep.mp3");
    std::fs::write(&audio, b"audio").unwrap();
    assert!(backup::write(&library, &audio).is_err());
    assert_eq!(std::fs::read(audio).unwrap(), b"audio");
}
#[test]
fn bulk_playlist_action_rolls_back_if_any_track_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let mut library = Library::open(&temp.path().join("library.sqlite")).unwrap();
    add(&mut library, &[temp.path().join("a.mp3")]);
    let id = library.full_snapshot().unwrap().songs[0].id;
    let pl = library.create_playlist("Mix").unwrap();
    assert!(library.add_many_to_playlist(pl, &[id, 99999]).is_err());
    assert!(
        library.full_snapshot().unwrap().playlists[0]
            .entries
            .is_empty()
    );
    library.set_favorites(&[id], true).unwrap();
    library.set_favorites(&[id], true).unwrap();
    assert!(library.full_snapshot().unwrap().songs[0].favorite);
}
#[test]
fn folder_browser_shows_direct_children_without_flattening_descendants() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut library = Library::open(&root.join("library.sqlite")).unwrap();
    library.add_root(root).unwrap();
    add(
        &mut library,
        &[
            root.join("one.mp3"),
            root.join("Album/two.mp3"),
            root.join("Album/Disc/three.mp3"),
        ],
    );
    let snapshot = library.snapshot().unwrap();
    let rows = browsing::build_rows(
        &snapshot,
        &BrowsePage::Folder(root.into()),
        "",
        &Queue::default(),
        &snapshot.preferences,
    );
    assert_eq!(rows.len(), 2);
    assert!(
        matches!(&rows[0],BrowseRow::Group{title,page:BrowsePage::Folder(path),..} if title=="Album" && path==&root.join("Album"))
    );
    assert!(matches!(&rows[1],BrowseRow::Song{song,..} if song.title=="one.mp3"));
    let rows = browsing::build_rows(
        &snapshot,
        &BrowsePage::Folder(root.join("Album")),
        "",
        &Queue::default(),
        &snapshot.preferences,
    );
    assert_eq!(rows.len(), 2);
}

#[test]
fn backup_restores_custom_artwork_and_maps_path_preferences() {
    let temp = tempfile::tempdir().unwrap();
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    let mut source = Library::open(&temp.path().join("source.sqlite")).unwrap();
    add(&mut source, &[old.join("a.mp3")]);
    let song = source.full_snapshot().unwrap().songs[0].id;
    let playlist = source.create_playlist("Art mix").unwrap();
    source.add_to_playlist(playlist, song).unwrap();
    let image = temp.path().join("custom.png");
    image::RgbImage::from_pixel(16, 16, image::Rgb([20, 40, 60]))
        .save(&image)
        .unwrap();
    source
        .set_artwork(&format!("playlist:{playlist}"), Some(&image))
        .unwrap();
    let mut preferences = source.full_snapshot().unwrap().preferences;
    preferences.excluded_folders.push(old.join("Hidden"));
    preferences.lyric_offsets.insert(old.join("a.mp3"), 300);
    source.save_preferences(&preferences).unwrap();
    let file = temp.path().join("backup.tremors-backup");
    backup::write(&source, &file).unwrap();
    let document = backup::read(&file).unwrap();
    let paths = AppPaths::at(temp.path().join("target")).unwrap();
    let mut target = Library::open(&paths.database).unwrap();
    add(&mut target, &[new.join("a.mp3")]);
    target.create_playlist("Existing").unwrap();
    let mut request = Restore {
        backup: document,
        remap: Some((old, new.clone())),
        preferences: true,
    };
    backup::restore(&mut target, &paths, &request).unwrap();
    let snapshot = target.full_snapshot().unwrap();
    let playlist = snapshot
        .playlists
        .iter()
        .find(|p| p.name == "Art mix")
        .unwrap();
    let restored = &snapshot.artwork[&format!("playlist:{}", playlist.id)];
    assert!(restored.starts_with(&paths.covers));
    assert_eq!(
        image::open(restored).unwrap().to_rgb8().get_pixel(0, 0),
        &image::Rgb([20, 40, 60])
    );
    assert_eq!(snapshot.preferences.excluded_folders, [new.join("Hidden")]);
    assert_eq!(snapshot.preferences.lyric_offsets[&new.join("a.mp3")], 300);
    request.backup.artwork[0].bytes = b"invalid image".to_vec();
    assert!(backup::restore(&mut target, &paths, &request).is_err());
    assert_eq!(target.full_snapshot().unwrap().playlists.len(), 2);
}

// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use tremors_core::{
    browsing::{BrowsePage, BrowseRow, build_rows, song_index, visible_songs},
    model::{Playlist, PlaylistEntry, Snapshot, Song},
    queue::Queue,
    settings::Preferences,
};

fn song(id: i64, title: &str, artist: &str) -> Song {
    Song {
        id,
        title: title.into(),
        artist: artist.into(),
        album: "Shared album".into(),
        album_artist: artist.into(),
        ..Default::default()
    }
}

fn rows(snapshot: &Snapshot, page: &BrowsePage, query: &str, queue: &Queue) -> Vec<BrowseRow> {
    build_rows(snapshot, page, query, queue, &Preferences::default())
}

#[test]
fn search_groups_do_not_shift_playable_song_selection() {
    let snapshot = Snapshot {
        songs: vec![song(1, "Blue sky", "Blue"), song(2, "Blue", "Other")],
        ..Default::default()
    };
    let results = rows(&snapshot, &BrowsePage::Search, "Blue", &Queue::default());
    assert!(matches!(results.first(), Some(BrowseRow::Group { .. })));
    assert_eq!(song_index(&results, 0), None);
    assert_eq!(song_index(&results, 1), None);
    assert_eq!(song_index(&results, 2), Some(0));
    assert_eq!(song_index(&results, 3), Some(1));
    assert_eq!(song_index(&results, usize::MAX), None);
    assert_eq!(
        visible_songs(&results)
            .iter()
            .map(|s| s.id)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
}

#[test]
fn filtered_queue_rows_keep_their_original_duplicate_positions() {
    let mut queue = Queue::default();
    queue.replace(
        vec![
            song(1, "Keep", "A"),
            song(2, "Other", "B"),
            song(1, "Keep", "A"),
        ],
        2,
    );
    let results = rows(&Snapshot::default(), &BrowsePage::Queue, "Keep", &queue);
    let positions: Vec<_> = results
        .iter()
        .map(|row| match row {
            BrowseRow::Song { queue_index, .. } => *queue_index,
            _ => panic!("queue results must be songs"),
        })
        .collect();
    assert_eq!(positions, vec![Some(0), Some(2)]);
    assert_eq!(visible_songs(&results).len(), 2);
    assert_eq!(queue.current_index(), Some(2));
}

#[test]
fn playlist_filtering_preserves_entry_identity_and_order() {
    let snapshot = Snapshot {
        songs: vec![song(1, "Keep", "A"), song(2, "Other", "B")],
        playlists: vec![Playlist {
            id: 10,
            name: "Duplicates".into(),
            entries: vec![
                PlaylistEntry { id: 21, song_id: 1 },
                PlaylistEntry { id: 22, song_id: 2 },
                PlaylistEntry { id: 23, song_id: 1 },
            ],
        }],
        ..Default::default()
    };
    let results = rows(
        &snapshot,
        &BrowsePage::Playlist(10),
        "Keep",
        &Queue::default(),
    );
    let entries: Vec<_> = results
        .iter()
        .map(|row| match row {
            BrowseRow::Song { entry, .. } => *entry,
            _ => panic!("playlist results must be songs"),
        })
        .collect();
    assert_eq!(entries, vec![Some(21), Some(23)]);
    assert_eq!(
        visible_songs(&results)
            .iter()
            .map(|s| s.id)
            .collect::<Vec<_>>(),
        vec![1, 1]
    );
}

#[test]
fn albums_with_the_same_title_keep_distinct_album_artists() {
    let snapshot = Snapshot {
        songs: vec![song(1, "A", "First"), song(2, "B", "Second")],
        ..Default::default()
    };
    let groups = rows(&snapshot, &BrowsePage::Albums, "", &Queue::default());
    assert_eq!(groups.len(), 2);
    let tracks = rows(
        &snapshot,
        &BrowsePage::Album("Shared album".into(), "Second".into()),
        "",
        &Queue::default(),
    );
    assert_eq!(visible_songs(&tracks)[0].id, 2);
}

#[test]
fn empty_search_and_missing_playlists_have_no_playable_results() {
    let snapshot = Snapshot {
        songs: vec![song(1, "One", "A")],
        ..Default::default()
    };
    for (page, query) in [(BrowsePage::Search, "  "), (BrowsePage::Playlist(99), "")] {
        let results = rows(&snapshot, &page, query, &Queue::default());
        assert!(results.is_empty());
        assert!(visible_songs(&results).is_empty());
    }
}

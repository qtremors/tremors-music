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

#[test]
fn home_shelves_rank_whole_albums_and_keep_album_artists_distinct() {
    use tremors_core::browsing::home_sections;
    let snapshot = Snapshot {
        songs: vec![
            Song {
                added_at: 10,
                play_count: 3,
                ..song(1, "One", "First")
            },
            Song {
                added_at: 30,
                play_count: 4,
                ..song(2, "Two", "First")
            },
            Song {
                added_at: 20,
                play_count: 6,
                ..song(3, "Three", "Second")
            },
            Song {
                album: "Unplayed".into(),
                added_at: 40,
                ..song(4, "Four", "Third")
            },
        ],
        ..Default::default()
    };
    let sections = home_sections(&snapshot);
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].rows.len(), 3);
    let pages = |section: usize| {
        sections[section]
            .rows
            .iter()
            .map(|row| match row {
                BrowseRow::Group { page, .. } => page.clone(),
                _ => panic!("home shelves must contain collections"),
            })
            .collect::<Vec<_>>()
    };
    assert!(
        pages(0)
            == vec![
                BrowsePage::Album("Unplayed".into(), "Third".into()),
                BrowsePage::Album("Shared album".into(), "First".into()),
                BrowsePage::Album("Shared album".into(), "Second".into()),
            ]
    );
    assert!(
        pages(1)
            == vec![
                BrowsePage::Album("Shared album".into(), "First".into()),
                BrowsePage::Album("Shared album".into(), "Second".into()),
            ]
    );
}

#[test]
fn home_shelves_are_bounded_and_do_not_fabricate_content() {
    use tremors_core::browsing::home_sections;
    assert!(home_sections(&Snapshot::default()).is_empty());
    let snapshot = Snapshot {
        songs: (0..50)
            .map(|id| Song {
                album: format!("Album {id}"),
                added_at: id,
                play_count: 1,
                ..song(id, "Track", "Artist")
            })
            .collect(),
        playlists: vec![Playlist {
            id: 9,
            name: "Empty mix".into(),
            entries: vec![],
        }],
        ..Default::default()
    };
    let sections = home_sections(&snapshot);
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].rows.len(), 8);
    assert_eq!(sections[1].rows.len(), 8);
    assert_eq!(sections[2].rows.len(), 1);
    assert!(matches!(
        &sections[2].rows[0],
        BrowseRow::Group {
            page: BrowsePage::Playlist(9),
            cover: None,
            ..
        }
    ));
    assert!(
        build_rows(
            &snapshot,
            &BrowsePage::Home,
            "",
            &Queue::default(),
            &Preferences::default()
        )
        .is_empty()
    );
}

#[test]
fn artist_album_navigation_includes_compilations_and_excludes_other_performers() {
    use tremors_core::browsing::artist_albums;
    let snapshot = Snapshot {
        songs: vec![
            Song {
                album_artist: "Various artists".into(),
                ..song(1, "Feature", "Guest")
            },
            Song {
                album_artist: "Various artists".into(),
                ..song(4, "Another performer", "Someone else")
            },
            Song {
                album: "Solo record".into(),
                ..song(2, "Solo", "Guest")
            },
            Song {
                album: "Different record".into(),
                ..song(3, "Other", "Someone else")
            },
        ],
        ..Default::default()
    };
    let albums = artist_albums(&snapshot, "Guest");
    assert_eq!(albums.len(), 2);
    for album in albums {
        let BrowseRow::Group { page, subtitle, .. } = album else {
            panic!("expected an album");
        };
        if matches!(&page, BrowsePage::Album(_, artist) if artist == "Various artists") {
            assert!(subtitle.ends_with("2 tracks"));
        }
        let tracks = visible_songs(&rows(&snapshot, &page, "", &Queue::default()));
        assert!(tracks.iter().any(|s| s.artist == "Guest"));
    }
    assert!(artist_albums(&snapshot, "Missing").is_empty());
}

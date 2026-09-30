// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use crate::{
    model::{Snapshot, Song},
    queue::Queue,
    settings::{Preferences, SongSort},
};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::Arc,
};

pub fn sort_songs(songs: &mut [Song], sort: SongSort, descending: bool) {
    songs.sort_by(|a, b| {
        let order = match sort {
            SongSort::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            SongSort::Artist => a.artist.to_lowercase().cmp(&b.artist.to_lowercase()),
            SongSort::Album => a
                .album
                .to_lowercase()
                .cmp(&b.album.to_lowercase())
                .then(a.disc.cmp(&b.disc))
                .then(a.track.cmp(&b.track)),
            SongSort::Duration => a.duration.total_cmp(&b.duration),
            SongSort::Year => a.year.cmp(&b.year),
            SongSort::Added => a.added_at.cmp(&b.added_at),
            SongSort::Size => a.details.file_size.cmp(&b.details.file_size),
        }
        .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        .then(a.id.cmp(&b.id));
        if descending { order.reverse() } else { order }
    });
}

pub fn search_rank(song: &Song, query: &str) -> u8 {
    let query = query.trim().to_lowercase();
    if song.title.to_lowercase() == query {
        0
    } else if song.title.to_lowercase().starts_with(&query) {
        1
    } else if song.artist.to_lowercase() == query || song.album.to_lowercase() == query {
        2
    } else if song.title.to_lowercase().contains(&query) {
        3
    } else {
        4
    }
}

/// Navigation targets shared by browsing state and desktop presentation.
#[derive(Clone, PartialEq, Eq)]
pub enum BrowsePage {
    Library,
    Albums,
    Artists,
    Genres,
    Genre(String),
    Playlists,
    Search,
    Settings,
    Lyrics,
    Favorites,
    Recent,
    MostPlayed,
    Playlist(i64),
    Album(String, String),
    Artist(String),
    Queue,
    Folders,
}

#[derive(Clone)]
pub enum BrowseRow {
    Song {
        song: Arc<Song>,
        entry: Option<i64>,
        queue_index: Option<usize>,
    },
    Group {
        title: String,
        subtitle: String,
        cover: Option<PathBuf>,
        page: BrowsePage,
    },
}

/// Build a view without UI state, I/O, or mutation of the source library.
pub fn build_rows(
    snapshot: &Snapshot,
    page: &BrowsePage,
    query: &str,
    queue: &Queue,
    preferences: &Preferences,
) -> Vec<BrowseRow> {
    use BrowsePage as Page;
    use BrowseRow as Row;
    let mut rows = Vec::new();
    let songs = &snapshot.songs;
    let by_id: HashMap<_, _> = songs.iter().map(|s| (s.id, s)).collect();
    match page {
        Page::Albums => {
            let mut groups: BTreeMap<(String, String), Vec<&Song>> = BTreeMap::new();
            for s in songs {
                groups
                    .entry((s.album.clone(), s.album_artist.clone()))
                    .or_default()
                    .push(s);
            }
            for ((album, artist), songs) in groups {
                if !format!("{album} {artist}")
                    .to_lowercase()
                    .contains(&query.to_lowercase())
                {
                    continue;
                }
                rows.push(Row::Group {
                    title: album.clone(),
                    subtitle: format!("{artist} · {} tracks", songs.len()),
                    cover: songs.iter().find_map(|s| s.cover.clone()),
                    page: Page::Album(album, artist),
                });
            }
        }
        Page::Artists => {
            let mut groups: BTreeMap<String, Vec<&Song>> = BTreeMap::new();
            for s in songs {
                groups.entry(s.artist.clone()).or_default().push(s);
            }
            for (artist, songs) in groups {
                if !artist.to_lowercase().contains(&query.to_lowercase()) {
                    continue;
                }
                rows.push(Row::Group {
                    title: artist.clone(),
                    subtitle: format!("{} tracks", songs.len()),
                    cover: songs.iter().find_map(|s| s.cover.clone()),
                    page: Page::Artist(artist),
                });
            }
        }
        Page::Genres => {
            let mut groups: BTreeMap<String, Vec<&Song>> = BTreeMap::new();
            for s in songs {
                groups
                    .entry(if s.genre.is_empty() {
                        "Unknown genre".into()
                    } else {
                        s.genre.clone()
                    })
                    .or_default()
                    .push(s);
            }
            for (genre, songs) in groups {
                if !genre.to_lowercase().contains(&query.to_lowercase()) {
                    continue;
                }
                rows.push(Row::Group {
                    title: genre.clone(),
                    subtitle: format!("{} tracks", songs.len()),
                    cover: songs.iter().find_map(|s| s.cover.clone()),
                    page: Page::Genre(genre),
                });
            }
        }
        Page::Playlists => {
            for pl in &snapshot.playlists {
                if !pl.name.to_lowercase().contains(&query.to_lowercase()) {
                    continue;
                }
                rows.push(Row::Group {
                    title: pl.name.clone(),
                    subtitle: format!("{} tracks", pl.entries.len()),
                    cover: pl
                        .entries
                        .iter()
                        .filter_map(|e| by_id.get(&e.song_id))
                        .find_map(|s| s.cover.clone()),
                    page: Page::Playlist(pl.id),
                });
            }
        }
        Page::Search => {
            if query.trim().is_empty() {
                return rows;
            }
            let q = query.to_lowercase();
            let mut albums = BTreeMap::new();
            let mut artists = BTreeMap::new();
            for s in songs {
                if format!("{} {}", s.album, s.album_artist)
                    .to_lowercase()
                    .contains(&q)
                {
                    albums
                        .entry((s.album.clone(), s.album_artist.clone()))
                        .or_insert(s.cover.clone());
                }
                if s.artist.to_lowercase().contains(&q) {
                    artists.entry(s.artist.clone()).or_insert(s.cover.clone());
                }
            }
            for (artist, cover) in artists {
                rows.push(Row::Group {
                    title: artist.clone(),
                    subtitle: "Artist".into(),
                    cover,
                    page: Page::Artist(artist),
                });
            }
            for ((album, artist), cover) in albums {
                rows.push(Row::Group {
                    title: album.clone(),
                    subtitle: format!("Album · {artist}"),
                    cover,
                    page: Page::Album(album, artist),
                });
            }
            let mut selected: Vec<_> = songs.iter().filter(|s| s.matches(query)).cloned().collect();
            selected.sort_by_cached_key(|s| (search_rank(s, query), s.title.to_lowercase()));
            rows.extend(selected.into_iter().map(|song| Row::Song {
                song: Arc::new(song),
                entry: None,
                queue_index: None,
            }));
        }
        Page::Playlist(id) => {
            if let Some(pl) = snapshot.playlists.iter().find(|pl| pl.id == *id) {
                for entry in &pl.entries {
                    if let Some(song) = by_id.get(&entry.song_id).filter(|s| s.matches(query)) {
                        rows.push(Row::Song {
                            song: Arc::new((**song).clone()),
                            entry: Some(entry.id),
                            queue_index: None,
                        });
                    }
                }
            }
        }
        Page::Queue => {
            for (index, song) in queue.ordered().into_iter().enumerate() {
                if song.matches(query) {
                    rows.push(Row::Song {
                        song: Arc::new(song),
                        entry: None,
                        queue_index: Some(index),
                    });
                }
            }
        }
        Page::Folders | Page::Settings | Page::Lyrics => {}
        page => {
            let mut selected: Vec<_> = songs
                .iter()
                .filter(|s| s.matches(query))
                .filter(|s| match page {
                    Page::Favorites => s.favorite,
                    Page::MostPlayed => s.play_count > 0,
                    Page::Album(album, artist) => s.album == *album && s.album_artist == *artist,
                    Page::Artist(artist) => s.artist == *artist,
                    Page::Genre(genre) => {
                        s.genre == *genre || (genre == "Unknown genre" && s.genre.is_empty())
                    }
                    _ => true,
                })
                .cloned()
                .collect();
            match page {
                Page::Recent => selected.sort_by_key(|s| std::cmp::Reverse((s.added_at, s.id))),
                Page::MostPlayed => selected.sort_by_key(|s| std::cmp::Reverse(s.play_count)),
                Page::Album(..) => selected
                    .sort_by_key(|s| (s.disc.unwrap_or(1), s.track.unwrap_or(0), s.title.clone())),
                _ => sort_songs(&mut selected, preferences.sort, preferences.descending),
            }
            rows = selected
                .into_iter()
                .map(|song| Row::Song {
                    song: Arc::new(song),
                    entry: None,
                    queue_index: None,
                })
                .collect();
        }
    }
    rows
}

/// Tracks in visible order, preserving duplicate playlist and queue entries.
pub fn visible_songs(rows: &[BrowseRow]) -> Vec<Song> {
    rows.iter()
        .filter_map(|row| match row {
            BrowseRow::Song { song, .. } => Some((**song).clone()),
            _ => None,
        })
        .collect()
}

/// Map a mixed result row to its position in the playable tracks.
pub fn song_index(rows: &[BrowseRow], index: usize) -> Option<usize> {
    matches!(rows.get(index), Some(BrowseRow::Song { .. })).then(|| {
        rows[..index]
            .iter()
            .filter(|row| matches!(row, BrowseRow::Song { .. }))
            .count()
    })
}

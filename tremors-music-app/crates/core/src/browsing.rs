// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use crate::{
    model::{Snapshot, Song, count_label},
    queue::Queue,
    settings::{Preferences, SongSort},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
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
    Home,
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
    RecentlyPlayed,
    Metadata(i64),
    LyricsEditor(i64),
    Playlist(i64),
    Album(String, String),
    Artist(String),
    Queue,
    Folders,
    FolderBrowser,
    Folder(PathBuf),
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
        Page::FolderBrowser => {
            // Roots are library navigation only: browsing never walks arbitrary disk paths.
            for root in &snapshot.roots {
                let count = songs.iter().filter(|s| s.path.starts_with(root)).count();
                rows.push(Row::Group {
                    title: root
                        .file_name()
                        .unwrap_or(root.as_os_str())
                        .to_string_lossy()
                        .into_owned(),
                    subtitle: format!("{} · {}", count_label(count, "song"), root.display()),
                    cover: songs
                        .iter()
                        .find(|s| s.path.starts_with(root))
                        .and_then(|s| s.cover.clone()),
                    page: Page::Folder(root.clone()),
                });
            }
        }
        Page::Folder(folder) => {
            let mut children = BTreeMap::<PathBuf, usize>::new();
            for song in songs {
                if let Ok(relative) = song.path.strip_prefix(folder) {
                    let parts: Vec<_> = relative.components().collect();
                    if parts.len() > 1 {
                        *children.entry(folder.join(parts[0])).or_default() += 1;
                    } else if song.matches(query) {
                        rows.push(Row::Song {
                            song: Arc::new(song.clone()),
                            entry: None,
                            queue_index: None,
                        });
                    }
                }
            }
            let mut folders: Vec<_> = children
                .into_iter()
                .filter(|(path, _)| {
                    path.file_name().is_some_and(|name| {
                        name.to_string_lossy()
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                    })
                })
                .map(|(path, count)| Row::Group {
                    title: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    subtitle: count_label(count, "song"),
                    cover: None,
                    page: Page::Folder(path),
                })
                .collect();
            folders.extend(rows);
            return folders;
        }
        Page::Albums => return album_rows(songs.iter(), query),
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
                    subtitle: count_label(songs.len(), "track"),
                    cover: snapshot
                        .artwork
                        .get(&format!("artist:{artist}"))
                        .cloned()
                        .or_else(|| songs.iter().find_map(|s| s.cover.clone())),
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
                    subtitle: count_label(songs.len(), "track"),
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
                    subtitle: count_label(
                        pl.entries
                            .iter()
                            .filter(|e| by_id.contains_key(&e.song_id))
                            .count(),
                        "track",
                    ),
                    cover: snapshot
                        .artwork
                        .get(&format!("playlist:{}", pl.id))
                        .cloned()
                        .or_else(|| {
                            pl.entries
                                .iter()
                                .filter_map(|e| by_id.get(&e.song_id))
                                .find_map(|s| s.cover.clone())
                        }),
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
        Page::Home
        | Page::Folders
        | Page::Settings
        | Page::Lyrics
        | Page::Metadata(_)
        | Page::LyricsEditor(_) => {}
        page => {
            let mut selected: Vec<_> = songs
                .iter()
                .filter(|s| s.matches(query))
                .filter(|s| match page {
                    Page::Favorites => s.favorite,
                    Page::MostPlayed => s.play_count > 0,
                    Page::RecentlyPlayed => s.last_played.is_some(),
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
                Page::RecentlyPlayed => {
                    selected.sort_by_key(|s| std::cmp::Reverse((s.last_played, s.id)))
                }
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

/// A bounded shelf of local collections for the library home page.
#[derive(Clone)]
pub struct HomeSection {
    pub title: &'static str,
    pub page: BrowsePage,
    pub rows: Vec<BrowseRow>,
}

pub fn home_sections(snapshot: &Snapshot) -> Vec<HomeSection> {
    let mut albums: BTreeMap<(String, String), Vec<&Song>> = BTreeMap::new();
    for song in &snapshot.songs {
        albums
            .entry((song.album.clone(), song.album_artist.clone()))
            .or_default()
            .push(song);
    }
    let mut recent = Vec::new();
    let mut rotation = Vec::new();
    for ((album, artist), songs) in albums {
        let row = BrowseRow::Group {
            title: album.clone(),
            subtitle: artist.clone(),
            cover: songs.iter().find_map(|song| song.cover.clone()),
            page: BrowsePage::Album(album, artist),
        };
        let added = songs
            .iter()
            .map(|s| (s.added_at, s.id))
            .max()
            .unwrap_or_default();
        let plays: u64 = songs.iter().map(|s| u64::from(s.play_count)).sum();
        recent.push((added, row.clone()));
        if plays > 0 {
            rotation.push((plays, row));
        }
    }
    recent.sort_by_key(|(added, _)| std::cmp::Reverse(*added));
    rotation.sort_by_key(|(plays, _)| std::cmp::Reverse(*plays));
    let mut sections = vec![
        HomeSection {
            title: "Recently added",
            page: BrowsePage::Recent,
            rows: recent.into_iter().take(8).map(|(_, row)| row).collect(),
        },
        HomeSection {
            title: "In your rotation",
            page: BrowsePage::MostPlayed,
            rows: rotation.into_iter().take(8).map(|(_, row)| row).collect(),
        },
    ];
    sections.push(HomeSection {
        title: "Made by you",
        page: BrowsePage::Playlists,
        rows: build_rows(
            snapshot,
            &BrowsePage::Playlists,
            "",
            &Queue::default(),
            &Preferences::default(),
        )
        .into_iter()
        .take(8)
        .collect(),
    });
    sections.retain(|section| !section.rows.is_empty());
    sections
}

fn album_rows<'a>(songs: impl Iterator<Item = &'a Song>, query: &str) -> Vec<BrowseRow> {
    let mut groups: BTreeMap<(String, String), Vec<&Song>> = BTreeMap::new();
    for song in songs {
        groups
            .entry((song.album.clone(), song.album_artist.clone()))
            .or_default()
            .push(song);
    }
    let query = query.to_lowercase();
    groups
        .into_iter()
        .filter(|((album, artist), _)| format!("{album} {artist}").to_lowercase().contains(&query))
        .map(|((album, artist), songs)| BrowseRow::Group {
            title: album.clone(),
            subtitle: format!("{artist} · {}", count_label(songs.len(), "track")),
            cover: songs.iter().find_map(|song| song.cover.clone()),
            page: BrowsePage::Album(album, artist),
        })
        .collect()
}

/// Include albums containing the selected performer, including compilation records.
pub fn artist_albums(snapshot: &Snapshot, artist: &str) -> Vec<BrowseRow> {
    let albums: HashSet<_> = snapshot
        .songs
        .iter()
        .filter(|song| song.artist == artist)
        .map(|song| (&song.album, &song.album_artist))
        .collect();
    album_rows(
        snapshot
            .songs
            .iter()
            .filter(|song| albums.contains(&(&song.album, &song.album_artist))),
        "",
    )
}

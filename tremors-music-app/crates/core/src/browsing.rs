// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use crate::{model::Song, settings::SongSort};

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

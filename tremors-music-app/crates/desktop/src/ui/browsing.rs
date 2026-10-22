// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Derive visible rows from library snapshots and queue state.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn visible_songs(&self) -> Vec<Song> {
        visible_songs(&self.rows)
    }

    pub(in crate::ui) fn rebuild(&mut self) {
        self.artist_collections = if let Page::Artist(artist) = &self.page {
            artist_albums(&self.snapshot, artist)
        } else {
            Vec::new()
        };
        self.rows = build_rows(
            &self.snapshot,
            &self.page,
            &self.query,
            &self.queue,
            &self.preferences,
        );
    }
}

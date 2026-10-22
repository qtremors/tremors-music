// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Song selection is scoped to a page; actions keep its visible order.
use super::*;
impl MusicApp {
    fn selected_tracks(&self) -> Vec<Song> {
        let mut seen = std::collections::HashSet::new();
        self.rows
            .iter()
            .filter_map(|row| match row {
                Row::Song { song, .. }
                    if self.selected_songs.contains(&song.id) && seen.insert(song.id) =>
                {
                    Some((**song).clone())
                }
                _ => None,
            })
            .collect()
    }

    pub(in crate::ui) fn render_selection(&self, cx: &mut Context<Self>) -> AnyElement {
        let songs = self.selected_tracks();
        let ids: Vec<_> = songs.iter().map(|s| s.id).collect();
        let favorites = ids.clone();
        let unfavorites = ids.clone();
        let disabled = songs.is_empty();
        div()
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .pb_3()
            .child(
                Button::new("select-all")
                    .ghost()
                    .small()
                    .label("Select all")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.selected_songs = this.visible_songs().iter().map(|s| s.id).collect();
                        cx.notify();
                    })),
            )
            .child(
                Button::new("select-none")
                    .ghost()
                    .small()
                    .label("Clear")
                    .disabled(disabled)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.selected_songs.clear();
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} selected", ids.len())),
            )
            .child(
                Button::new("queue-selected")
                    .small()
                    .label("Queue")
                    .disabled(disabled)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        for song in &songs {
                            this.queue.append(song.clone());
                        }
                        this.persist_queue();
                        this.rebuild();
                        cx.notify();
                    })),
            )
            .child(
                Button::new("playlist-selected")
                    .small()
                    .label("Add to playlist")
                    .disabled(disabled)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_overlay(Overlay::AddManyToPlaylist(ids.clone()), window, cx);
                        cx.notify();
                    })),
            )
            .child(
                Button::new("favorite-selected")
                    .ghost()
                    .small()
                    .label("Favorite")
                    .disabled(disabled)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.send_library(LibraryCommand::SetFavorites(favorites.clone(), true));
                        cx.notify();
                    })),
            )
            .child(
                Button::new("unfavorite-selected")
                    .ghost()
                    .small()
                    .label("Unfavorite")
                    .disabled(disabled)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.send_library(LibraryCommand::SetFavorites(unfavorites.clone(), false));
                        cx.notify();
                    })),
            )
            .into_any_element()
    }
}

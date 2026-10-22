// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Page-specific empty states and virtualized collection galleries and tables.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_content(&mut self, cx: &mut Context<Self>) -> AnyElement {
        if matches!(self.page, Page::Metadata(_)) {
            return self.render_metadata(cx);
        }
        if self.page == Page::Settings {
            return self.render_settings(cx);
        }
        if self.page == Page::Lyrics {
            return self.render_lyrics(cx);
        }
        if self.page == Page::Folders {
            return self.render_folders(cx);
        }
        if self.snapshot.songs.is_empty() && self.page != Page::Playlists {
            if !self.snapshot.roots.is_empty()
                && (self.preferences.minimum_duration > 0
                    || !self.preferences.excluded_folders.is_empty())
            {
                return self.render_empty(
                    "No tracks match your filters",
                    "Change the duration or excluded folders in Music folders.",
                    IconName::Folder,
                    false,
                    cx,
                );
            }
            return self.render_empty(
                "Your music belongs here",
                "Add a music folder and bring your albums, artists, and favorite songs together.",
                IconName::Music,
                true,
                cx,
            );
        }
        if self.page == Page::Home {
            return self.render_home(cx);
        }
        if self.page == Page::Search && self.query.trim().is_empty() {
            return self.render_search_start(cx);
        }
        if self.rows.is_empty() {
            let (title, description, icon) = match self.page {
                Page::Favorites => (
                    "A little love goes a long way",
                    "Use the heart beside a song to keep it in your favorites.",
                    IconName::Heart,
                ),
                Page::Queue => (
                    "A fresh start",
                    "Play an album or add songs to your queue. They will appear here.",
                    IconName::ListMusic,
                ),
                Page::Playlists => (
                    "Make something your own",
                    "Create a playlist, then add songs using a track's menu.",
                    IconName::ListMusic,
                ),
                Page::Playlist(_) => (
                    "Your next great mix starts here",
                    "Browse Songs and use a track's menu to add it to this playlist.",
                    IconName::ListMusic,
                ),
                Page::RecentlyPlayed => (
                    "Nothing played yet",
                    "Play something from your library to find it here.",
                    IconName::Clock,
                ),
                Page::MostPlayed => (
                    "Your favorites are taking shape",
                    "Listen to your library and your most played songs will appear here.",
                    IconName::TrendingUp,
                ),
                Page::Search => (
                    "No matches in your library",
                    "Try another song, album, artist, or genre.",
                    IconName::Search,
                ),
                _ => (
                    "Nothing here yet",
                    "Add more music or explore another part of your collection.",
                    IconName::Music,
                ),
            };
            return self.render_empty(title, description, icon, false, cx);
        }
        let artist_gallery = matches!(self.page, Page::Artist(_)) && self.show_artist_albums;
        if artist_gallery
            || matches!(
                self.page,
                Page::Albums | Page::Artists | Page::Genres | Page::Playlists
            )
        {
            let view = cx.entity().downgrade();
            let columns = self.gallery_columns;
            let gallery = uniform_list(
                "gallery",
                if artist_gallery {
                    self.artist_collections.len()
                } else {
                    self.rows.len()
                }
                .div_ceil(columns),
                move |range, window, cx| {
                    let width = ((f32::from(window.viewport_size().width)
                        - 224.
                        - 64.
                        - (columns - 1) as f32 * 12.)
                        / columns as f32)
                        .max(100.);
                    view.update(cx, |this, cx| {
                        range
                            .map(|row| {
                                let mut strip =
                                    div().w_full().h(px(width + 86.)).px_6().flex().gap_3();
                                for col in 0..columns {
                                    let index = row * columns + col;
                                    strip = strip.child(
                                        if artist_gallery && index < this.artist_collections.len() {
                                            this.render_collection_card(
                                                this.artist_collections[index].clone(),
                                                format!("artist-album-{index}"),
                                                cx,
                                            )
                                        } else if !artist_gallery && index < this.rows.len() {
                                            this.render_card(index, cx)
                                        } else {
                                            div().flex_1().min_w_0().p_2().into_any_element()
                                        },
                                    );
                                }
                                strip.into_any_element()
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
                },
            )
            .track_scroll(&self.gallery_scroll)
            .flex_1()
            .min_h_0()
            .w_full();
            div()
                .relative()
                .flex_1()
                .min_h_0()
                .flex()
                .child(gallery)
                .child(Scrollbar::vertical(&self.gallery_scroll))
                .into_any_element()
        } else {
            let view = cx.entity().downgrade();
            let list = uniform_list("music-list", self.rows.len(), move |range, window, cx| {
                view.update(cx, |this, cx| {
                    range
                        .map(|i| this.render_row(i, window, cx))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
            })
            .track_scroll(&self.table_scroll)
            .flex_1()
            .min_h_0()
            .w_full();
            let mut body = div().flex_1().min_h_0().flex().flex_col().px_8().pb_4();
            if self.page != Page::Search {
                body = body.child(
                    div()
                        .h(px(32.))
                        .flex_shrink_0()
                        .px_2()
                        .flex()
                        .items_center()
                        .gap_3()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(div().w(px(36.)).text_center().child("#"))
                        .when(self.preferences.show_artwork, |d| d.child(div().w(px(36.))))
                        .child(div().flex_1().child("Title"))
                        .when(
                            !matches!(self.page, Page::Album(..) | Page::Queue | Page::Playlist(_)),
                            |d| {
                                d.child(div().w(px(120.)).child("Artist"))
                                    .child(div().w(px(140.)).child("Album"))
                            },
                        )
                        .child(div().w(px(40.)).text_right().child("Time"))
                        .child(div().w(px(
                            if matches!(self.page, Page::Queue | Page::Playlist(_)) {
                                168.
                            } else {
                                60.
                            },
                        ))),
                );
            }
            body.child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(list)
                    .child(Scrollbar::vertical(&self.table_scroll)),
            )
            .into_any_element()
        }
    }

    fn render_empty(
        &self,
        title: &str,
        description: &str,
        icon: IconName,
        add_folder: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .p_6()
            .child(
                div()
                    .size(px(72.))
                    .rounded_xl()
                    .bg(cx.theme().sidebar)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon).size_12().text_color(cx.theme().primary)),
            )
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child(title.to_owned()),
            )
            .child(
                div()
                    .max_w(px(400.))
                    .text_center()
                    .text_color(cx.theme().muted_foreground)
                    .child(description.to_owned()),
            )
            .when(add_folder, |d| {
                d.child(
                    Button::new("empty-add-folder")
                        .primary()
                        .label("Add music folder")
                        .icon(Icon::new(IconName::Plus))
                        .disabled(self.picking_folder || self.scanning)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.pick_folder();
                            cx.notify();
                        })),
                )
            })
            .into_any_element()
    }
}

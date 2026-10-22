// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Navigation toolbar and collection headers.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn play_visible(&mut self, shuffle: bool, cx: &mut Context<Self>) {
        let songs = self.visible_songs();
        if songs.is_empty() {
            return;
        }
        self.queue.set_shuffle(false);
        self.queue.replace(songs, 0);
        self.queue.set_shuffle(shuffle);
        self.persist_modes();
        self.load_current();
        self.rebuild();
        cx.notify();
    }

    pub(in crate::ui) fn render_header(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let toolbar = div()
            .h(px(52.))
            .flex_shrink_0()
            .px_8()
            .flex()
            .items_center()
            .gap_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                Button::new("back")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::ChevronLeft))
                    .tooltip("Back · Alt+Left")
                    .disabled(self.history.is_empty())
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.go_back(window, cx);
                    })),
            )
            .child(
                div().w(px(300.)).child(
                    Input::new(&self.search)
                        .small()
                        .disabled(matches!(self.page, Page::Metadata(_))),
                ),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("YOUR LOCAL LIBRARY"),
            )
            .child(
                Button::new("toolbar-add-folder")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Plus))
                    .tooltip("Add music folder")
                    .disabled(self.picking_folder || self.scanning)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.pick_folder();
                        cx.notify();
                    })),
            );
        let songs: Vec<_> = self
            .rows
            .iter()
            .filter_map(|row| {
                if let Row::Song { song, .. } = row {
                    Some(song)
                } else {
                    None
                }
            })
            .collect();
        let count = songs.len();
        let total: f64 = songs.iter().map(|song| song.duration).sum();
        let detail = matches!(
            self.page,
            Page::Album(..) | Page::Artist(_) | Page::Playlist(_)
        );
        let track_count = count_label(count, "song");
        let description = match &self.page {
            Page::Home => "Your music. Always within reach.".to_owned(),
            Page::Search if self.query.trim().is_empty() => {
                "Find something you love, right here on your device.".into()
            }
            Page::Search => format!("{} in your library", count_label(self.rows.len(), "result")),
            Page::Albums => format!(
                "{} in your collection",
                count_label(self.rows.len(), "album")
            ),
            Page::Artists => format!(
                "{} in your collection",
                count_label(self.rows.len(), "artist")
            ),
            Page::Genres => format!("{} to explore", count_label(self.rows.len(), "genre")),
            Page::Playlists => format!(
                "{} · Curated by you",
                count_label(self.snapshot.playlists.len(), "playlist")
            ),
            Page::Favorites => format!("{} · The songs you love", count_label(count, "favorite")),
            Page::Recent => format!("{track_count} · The latest in your library"),
            Page::Metadata(_) => "Tags, lyrics and artwork in your local files.".into(),
            Page::RecentlyPlayed => format!("{track_count} · Back to what you love"),
            Page::MostPlayed => format!("{track_count} · Worth another listen"),
            Page::Folders => "Make room for your collection.".into(),
            Page::Settings => "Make yourself at home.".into(),
            Page::Lyrics => "Every word, from your own music files.".into(),
            Page::Album(..) => {
                let mut metadata = Vec::new();
                if let Some(song) = songs.first() {
                    if !song.genre.is_empty() {
                        metadata.push(song.genre.clone());
                    }
                    if let Some(year) = song.year {
                        metadata.push(year.to_string());
                    }
                }
                metadata.push(track_count);
                metadata.push(duration_label(total));
                metadata.join(" · ")
            }
            Page::Artist(_) => format!(
                "{} · {track_count} in your library",
                count_label(self.artist_collections.len(), "album")
            ),
            _ => format!("{track_count} · {} total", duration_label(total)),
        };
        let mut actions = div().flex().items_center().gap_2();
        if count > 0 {
            actions = actions
                .child(
                    Button::new("play-all")
                        .primary()
                        .label("Play")
                        .icon(Icon::new(IconName::Play))
                        .on_click(cx.listener(|this, _, _, cx| this.play_visible(false, cx))),
                )
                .child(
                    Button::new("shuffle-all")
                        .label("Shuffle")
                        .icon(Icon::new(IconName::Shuffle))
                        .on_click(cx.listener(|this, _, _, cx| this.play_visible(true, cx))),
                );
        }
        if matches!(
            self.page,
            Page::Library | Page::Favorites | Page::Artist(_) | Page::Genre(_)
        ) && count > 0
            && !(matches!(self.page, Page::Artist(_)) && self.show_artist_albums)
        {
            let view = cx.entity().downgrade();
            actions = actions.child(div().flex_1()).child(
                Button::new("sort")
                    .ghost()
                    .small()
                    .label(format!(
                        "{} {}",
                        self.preferences.sort.label(),
                        if self.preferences.descending {
                            "↓"
                        } else {
                            "↑"
                        }
                    ))
                    .dropdown_menu(move |mut menu, _, _| {
                        for sort in SongSort::ALL {
                            let view = view.clone();
                            menu = menu.item(PopupMenuItem::new(sort.label()).on_click(
                                move |_, _, cx| {
                                    let _ = view.update(cx, |this, cx| {
                                        this.preferences.sort = sort;
                                        this.persist_modes();
                                        this.rebuild();
                                        cx.notify();
                                    });
                                },
                            ));
                        }
                        let view = view.clone();
                        menu.item(
                            PopupMenuItem::new("Reverse order").on_click(move |_, _, cx| {
                                let _ = view.update(cx, |this, cx| {
                                    this.preferences.descending = !this.preferences.descending;
                                    this.persist_modes();
                                    this.rebuild();
                                    cx.notify();
                                });
                            }),
                        )
                    }),
            );
        }
        if self.page == Page::Playlists {
            actions = actions.child(
                Button::new("import-playlist")
                    .label("Import M3U")
                    .disabled(self.picking_file)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.pick_file(super::super::file_tasks::FileTask::ImportPlaylist);
                        cx.notify();
                    })),
            );
            actions = actions.child(
                Button::new("page-new-playlist")
                    .primary()
                    .label("New playlist")
                    .icon(Icon::new(IconName::Plus))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.playlist_name.update(cx, |input, cx| {
                            input.set_value("", window, cx);
                            input.focus(window, cx);
                        });
                        this.overlay = Some(Overlay::CreatePlaylist);
                        cx.notify();
                    })),
            );
        }
        if self.page == Page::Queue {
            actions = actions.child(
                Button::new("clear-queue")
                    .ghost()
                    .label("Clear queue")
                    .disabled(count == 0)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.queue.clear();
                        this.stop();
                        this.rebuild();
                        cx.notify();
                    })),
            );
        }
        if matches!(self.page, Page::Artist(_)) {
            actions = actions
                .when(self.show_artist_albums, |row| row.child(div().flex_1()))
                .child(
                    Button::new("artist-albums-tab")
                        .ghost()
                        .small()
                        .label("Albums")
                        .selected(self.show_artist_albums)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_artist_albums = true;
                            this.page_revision += 1;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("artist-songs-tab")
                        .ghost()
                        .small()
                        .label("Songs")
                        .selected(!self.show_artist_albums)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_artist_albums = false;
                            this.page_revision += 1;
                            cx.notify();
                        })),
                );
        }
        if let Page::Playlist(id) = self.page {
            let view = cx.entity().downgrade();
            actions = actions.child(div().flex_1()).child(
                Button::new("playlist-options")
                    .ghost()
                    .icon(Icon::new(IconName::Ellipsis))
                    .tooltip("Playlist options")
                    .dropdown_menu(move |menu, _, _| {
                        let rename_view = view.clone();
                        let delete_view = view.clone();
                        menu.item(PopupMenuItem::new("Rename playlist").on_click(
                            move |_, window, cx| {
                                let _ = rename_view.update(cx, |this, cx| {
                                    let name = this
                                        .snapshot
                                        .playlists
                                        .iter()
                                        .find(|p| p.id == id)
                                        .map(|p| p.name.clone())
                                        .unwrap_or_default();
                                    this.playlist_name.update(cx, |input, cx| {
                                        input.set_value(name, window, cx);
                                        input.focus(window, cx);
                                    });
                                    this.overlay = Some(Overlay::RenamePlaylist(id));
                                    cx.notify();
                                });
                            },
                        ))
                        .item(
                            PopupMenuItem::new("Delete playlist").on_click(move |_, _, cx| {
                                let _ = delete_view.update(cx, |this, cx| {
                                    this.overlay = Some(Overlay::DeletePlaylist(id));
                                    cx.notify();
                                });
                            }),
                        )
                    }),
            );
        }
        if let Page::Playlist(id) = self.page {
            actions = actions.child(
                Button::new("export-playlist")
                    .ghost()
                    .small()
                    .label("Export M3U")
                    .disabled(self.picking_file)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.pick_file(super::super::file_tasks::FileTask::ExportPlaylist(
                            id, true,
                        ));
                        cx.notify();
                    })),
            );
        }
        let artwork_key = match &self.page {
            Page::Artist(name) => Some(format!("artist:{name}")),
            Page::Playlist(id) => Some(format!("playlist:{id}")),
            _ => None,
        };
        if let Some(key) = artwork_key.clone() {
            let reset_key = key.clone();
            actions = actions.child(
                Button::new("choose-collection-art")
                    .ghost()
                    .small()
                    .label("Artwork")
                    .disabled(self.picking_file)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.pick_file(super::super::file_tasks::FileTask::Artwork(key.clone()));
                        cx.notify();
                    })),
            );
            if self.snapshot.artwork.contains_key(&reset_key) {
                actions = actions.child(
                    Button::new("reset-collection-art")
                        .ghost()
                        .small()
                        .label("Reset art")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.send_library(LibraryCommand::SetArtwork(reset_key.clone(), None));
                            cx.notify();
                        })),
                );
            }
        }
        let mut heading = div().flex_1().min_w_0().flex().flex_col().gap_2();
        if detail {
            let kind = match self.page {
                Page::Album(..) => "ALBUM",
                Page::Artist(_) => "ARTIST",
                _ => "PLAYLIST",
            };
            heading = heading.child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(cx.theme().primary)
                    .child(kind),
            );
        }
        heading = heading.child(
            div()
                .text_3xl()
                .font_weight(FontWeight::BOLD)
                .child(self.title()),
        );
        if let Page::Album(_, artist) = &self.page {
            let artist = artist.clone();
            heading = heading.child(
                div()
                    .id("album-artist-link")
                    .cursor_pointer()
                    .text_lg()
                    .text_color(cx.theme().primary)
                    .child(artist.clone())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate(Page::Artist(artist.clone()), window, cx)
                    })),
            );
        }
        heading = heading
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(description),
            )
            .child(div().mt_3().child(actions));
        let cover = artwork_key
            .and_then(|key| self.snapshot.artwork.get(&key).cloned())
            .or_else(|| songs.iter().find_map(|song| song.cover.clone()));
        let mut hero = div().px_8().pt_6().pb_5().flex().items_center().gap_6();
        if detail {
            hero = hero.child(
                div()
                    .w(px(if self.gallery_columns < 4 { 140. } else { 184. }))
                    .flex_shrink_0()
                    .child(collection_artwork(
                        cover,
                        matches!(self.page, Page::Artist(_)),
                        cx,
                    )),
            );
        }
        div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(hero.child(heading))
            .into_any_element()
    }
}

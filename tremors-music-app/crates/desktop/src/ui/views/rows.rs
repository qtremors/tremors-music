// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Song rows, group cards, context actions, and drag-and-drop.

use super::*;

impl MusicApp {
    fn play_row(
        &mut self,
        index: usize,
        queue_index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        if let Some(qindex) = queue_index {
            self.queue.select(qindex);
        } else {
            let Some(selected) = song_index(&self.rows, index) else {
                return;
            };
            self.queue.replace(self.visible_songs(), selected);
        }
        self.load_current();
        self.rebuild();
        cx.notify();
    }

    pub(in crate::ui) fn render_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(row) = self.rows.get(index).cloned() else {
            return div().into_any_element();
        };
        match row {
            Row::Group {
                title,
                subtitle,
                cover,
                page,
            } => {
                let focus = self.collection_focus(
                    &format!("table-{}-{index}", self.page_revision),
                    super::super::scrolling::FocusTarget::Table(index),
                    window,
                    cx,
                );
                let activated = page.clone();
                div()
                    .id(("group", index))
                    .track_focus(&focus)
                    .tab_stop(true)
                    .key_context("CollectionCard")
                    .role(gpui_kit::accesskit::Role::Button)
                    .aria_label(format!("Open {title}"))
                    .focus_visible(|d| {
                        d.bg(cx.theme().muted)
                            .border_1()
                            .border_color(cx.theme().primary)
                    })
                    .on_action(
                        cx.listener(move |this, _: &ActivateCollection, window, cx| {
                            this.navigate(activated.clone(), window, cx)
                        }),
                    )
                    .w_full()
                    .h(px(52.))
                    .px_4()
                    .flex()
                    .items_center()
                    .gap_4()
                    .cursor_pointer()
                    .hover(|s| s.bg(cx.theme().muted))
                    .child(cover_element(cover, 36., cx))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(subtitle),
                            ),
                    )
                    .child(Icon::new(IconName::ChevronRight))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate(page.clone(), window, cx)
                    }))
                    .into_any_element()
            }
            Row::Song {
                song,
                entry,
                queue_index,
            } => {
                let menu_song = song.clone();
                let id = song.id;
                let playing = if let Some(qindex) = queue_index {
                    self.queue.current_index() == Some(qindex)
                } else {
                    self.queue.current().is_some_and(|s| s.id == id)
                };
                let mut row = div()
                    .id(("track", index))
                    .w_full()
                    .h(px(52.))
                    .px_2()
                    .flex()
                    .items_center()
                    .gap_3()
                    .rounded_md()
                    .when(index.is_multiple_of(2), |s| s.bg(cx.theme().sidebar))
                    .hover(|s| s.bg(cx.theme().muted))
                    .when(playing, |s| s.bg(cx.theme().muted));
                row = row
                    .when(self.selection_mode, |row| {
                        row.child(
                            Checkbox::new(("select-song", index))
                                .accessibility_label(format!("Select {}", song.title))
                                .checked(self.selected_songs.contains(&id))
                                .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                                    if *checked {
                                        this.selected_songs.insert(id);
                                    } else {
                                        this.selected_songs.remove(&id);
                                    }
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        Button::new(("play-track", index))
                            .ghost()
                            .small()
                            .w(px(36.))
                            .px_0()
                            .accessibility_label(format!("Play {}", song.title))
                            .when(playing, |b| {
                                b.icon(Icon::new(IconName::Volume2).text_color(cx.theme().primary))
                            })
                            .when(!playing, |b| {
                                let number = if matches!(self.page, Page::Album(..)) {
                                    song.track.unwrap_or(index as u32 + 1)
                                } else if self.page == Page::Search {
                                    song_index(&self.rows, index).unwrap_or(index) as u32 + 1
                                } else {
                                    index as u32 + 1
                                };
                                b.child(
                                    div()
                                        .w_full()
                                        .text_center()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(number.to_string()),
                                )
                            })
                            .tooltip("Play this track")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.play_row(index, queue_index, window, cx);
                            })),
                    )
                    .when(self.preferences.show_artwork, |row| {
                        row.child(cover_element(song.cover.clone(), 36., cx))
                    })
                    .child(
                        div()
                            .id(("track-title", index))
                            .flex_1()
                            .min_w_0()
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                                if event.click_count() == 2 {
                                    this.play_row(index, queue_index, window, cx);
                                }
                            }))
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .when(playing, |d| d.text_color(cx.theme().primary))
                                    .child(song.title.clone()),
                            )
                            .when(
                                matches!(self.page, Page::Search | Page::Queue | Page::Playlist(_)),
                                |d| {
                                    d.child(
                                        div()
                                            .truncate()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!("{} · {}", song.artist, song.album)),
                                    )
                                },
                            ),
                    )
                    .when(
                        !matches!(
                            self.page,
                            Page::Album(..) | Page::Search | Page::Queue | Page::Playlist(_)
                        ),
                        |row| {
                            let artist = song.artist.clone();
                            let album = song.album.clone();
                            let album_artist = song.album_artist.clone();
                            row.child(
                                div()
                                    .id(("track-artist", index))
                                    .w(px(120.))
                                    .truncate()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .cursor_pointer()
                                    .child(song.artist.clone())
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.navigate(Page::Artist(artist.clone()), window, cx)
                                    })),
                            )
                            .child(
                                div()
                                    .id(("track-album", index))
                                    .w(px(140.))
                                    .truncate()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .cursor_pointer()
                                    .child(song.album.clone())
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.navigate(
                                            Page::Album(album.clone(), album_artist.clone()),
                                            window,
                                            cx,
                                        )
                                    })),
                            )
                        },
                    )
                    .child(
                        div()
                            .w(px(40.))
                            .text_right()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(duration_label(song.duration)),
                    )
                    .child(
                        Button::new(("favorite", index))
                            .ghost()
                            .small()
                            .icon(
                                Icon::new(IconName::Heart).when(song.favorite, |icon| {
                                    icon.text_color(cx.theme().primary)
                                }),
                            )
                            .selected(song.favorite)
                            .toggled(song.favorite)
                            .tooltip(if song.favorite {
                                "Remove favorite"
                            } else {
                                "Favorite"
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.send_library(LibraryCommand::Favorite(id));
                                cx.notify();
                            })),
                    );
                if let (Page::Playlist(playlist), Some(entry)) = (self.page.clone(), entry) {
                    row = row
                        .child(
                            Button::new(("up", index))
                                .ghost()
                                .small()
                                .icon(Icon::new(IconName::ChevronUp))
                                .tooltip("Move up")
                                .on_click(cx.listener(move |this, _, _, _| {
                                    this.send_library(LibraryCommand::MoveEntry(
                                        playlist, entry, -1,
                                    ))
                                })),
                        )
                        .child(
                            Button::new(("down", index))
                                .ghost()
                                .small()
                                .icon(Icon::new(IconName::ChevronDown))
                                .tooltip("Move down")
                                .on_click(cx.listener(move |this, _, _, _| {
                                    this.send_library(LibraryCommand::MoveEntry(playlist, entry, 1))
                                })),
                        )
                        .child(
                            Button::new(("remove", index))
                                .ghost()
                                .small()
                                .icon(Icon::new(IconName::X))
                                .tooltip("Remove this playlist entry")
                                .on_click(cx.listener(move |this, _, _, _| {
                                    this.send_library(LibraryCommand::RemoveEntry(entry))
                                })),
                        );
                }
                if let Some(qindex) = queue_index {
                    for (label, delta, icon) in [
                        ("queue-up", -1, IconName::ChevronUp),
                        ("queue-down", 1, IconName::ChevronDown),
                    ] {
                        row = row.child(
                            Button::new((label, index))
                                .ghost()
                                .small()
                                .icon(Icon::new(icon))
                                .tooltip(if delta < 0 { "Move up" } else { "Move down" })
                                .disabled(
                                    (delta < 0 && qindex == 0)
                                        || (delta > 0 && qindex + 1 >= self.queue.len()),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.queue
                                        .move_to(qindex, qindex.saturating_add_signed(delta));
                                    this.persist_queue();
                                    this.rebuild();
                                    cx.notify();
                                })),
                        );
                    }
                }
                if let Some(qindex) = queue_index {
                    row = row.child(
                        Button::new(("queue-remove", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::X))
                            .tooltip("Remove from queue")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let was_playing = this.is_playing || this.loading;
                                let current_removed = this.queue.remove(qindex);
                                if current_removed {
                                    if this.queue.is_empty() || !was_playing {
                                        this.stop();
                                    } else {
                                        this.load_current();
                                    }
                                }
                                this.persist_queue();
                                this.rebuild();
                                cx.notify();
                            })),
                    );
                }
                let playlist = if let (Page::Playlist(pl), Some(entry)) = (&self.page, entry) {
                    Some((*pl, entry))
                } else {
                    None
                };
                if queue_index.is_some() || playlist.is_some() {
                    let drag = RowDrag {
                        queue_index,
                        playlist,
                        title: menu_song.title.clone(),
                    };
                    row = row
                        .on_drag(drag, |info: &RowDrag, _, _, cx| cx.new(|_| info.clone()))
                        .on_drop(cx.listener(move |this, drag: &RowDrag, _, cx| {
                            if let (Some(from), Some(to)) = (drag.queue_index, queue_index) {
                                this.queue.move_to(from, to);
                                this.persist_queue();
                                this.rebuild();
                            } else if let (Some((pl, entry)), Some((target_pl, target_entry))) =
                                (drag.playlist, playlist)
                                && pl == target_pl
                                && let Some(playlist) =
                                    this.snapshot.playlists.iter().find(|p| p.id == pl)
                            {
                                let from = playlist.entries.iter().position(|e| e.id == entry);
                                let to = playlist.entries.iter().position(|e| e.id == target_entry);
                                if let (Some(from), Some(to)) = (from, to) {
                                    this.send_library(LibraryCommand::MoveEntry(
                                        pl,
                                        entry,
                                        to as isize - from as isize,
                                    ));
                                }
                            }
                            cx.notify();
                        }));
                }
                let view = cx.entity().downgrade();
                let build_menu =
                    move |mut menu: PopupMenu, _: &mut Window, _: &mut Context<PopupMenu>| {
                        for label in [
                            "Play next",
                            "Add to queue",
                            "Favorite / unfavorite",
                            "Add to playlist",
                            "Go to album",
                            "Go to artist",
                            "Edit metadata",
                            "Track details",
                        ] {
                            let view = view.clone();
                            let song = menu_song.clone();
                            menu = menu.item(PopupMenuItem::new(label).on_click(
                                move |_, window, cx| {
                                    let _ = view.update(cx, |this, cx| {
                                        match label {
                                            "Edit metadata" => {
                                                this.open_metadata(song.id, window, cx)
                                            }
                                            "Play next" => {
                                                this.queue.play_next((*song).clone());
                                                this.persist_queue();
                                                this.status = "Added to play next.".into();
                                                this.rebuild();
                                            }
                                            "Add to queue" => {
                                                this.queue.append((*song).clone());
                                                this.persist_queue();
                                                this.rebuild();
                                            }
                                            "Favorite / unfavorite" => {
                                                this.send_library(LibraryCommand::Favorite(song.id))
                                            }
                                            "Add to playlist" => this.open_overlay(
                                                Overlay::AddToPlaylist(song.id),
                                                window,
                                                cx,
                                            ),
                                            "Go to album" => this.navigate(
                                                Page::Album(
                                                    song.album.clone(),
                                                    song.album_artist.clone(),
                                                ),
                                                window,
                                                cx,
                                            ),
                                            "Go to artist" => this.navigate(
                                                Page::Artist(song.artist.clone()),
                                                window,
                                                cx,
                                            ),
                                            _ => this.open_overlay(
                                                Overlay::Details(Box::new((*song).clone())),
                                                window,
                                                cx,
                                            ),
                                        }
                                        cx.notify();
                                    });
                                },
                            ));
                        }
                        menu
                    };
                row.child(
                    Button::new(("track-menu", index))
                        .ghost()
                        .small()
                        .icon(Icon::new(IconName::Ellipsis))
                        .tooltip("Track actions")
                        .dropdown_menu(build_menu.clone()),
                )
                .context_menu(build_menu)
                .into_any_element()
            }
        }
    }

    pub(in crate::ui) fn render_card(
        &self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(row) = self.rows.get(index).cloned() else {
            return div().flex_1().into_any_element();
        };
        self.render_collection_card(
            row,
            format!("card-{index}"),
            scrolling::FocusTarget::Gallery(index),
            window,
            cx,
        )
    }

    pub(in crate::ui) fn render_collection_card(
        &self,
        row: Row,
        key: String,
        target: scrolling::FocusTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Row::Group {
            title,
            subtitle,
            cover,
            page,
        } = row
        else {
            return div().into_any_element();
        };
        let artist = matches!(page, Page::Artist(_));
        let artwork = if let Page::Playlist(id) = page {
            let covers: Vec<_> = self
                .snapshot
                .playlists
                .iter()
                .find(|pl| pl.id == id)
                .into_iter()
                .flat_map(|pl| &pl.entries)
                .filter_map(|entry| self.snapshot.songs.iter().find(|s| s.id == entry.song_id))
                .take(4)
                .map(|s| s.cover.clone())
                .collect();
            if covers.len() > 1
                && !self
                    .snapshot
                    .artwork
                    .contains_key(&format!("playlist:{id}"))
            {
                let mut collage = div()
                    .w_full()
                    .aspect_ratio(1.)
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .rounded_lg();
                for row in 0..2 {
                    collage =
                        collage.child(div().flex().flex_1().min_h_0().children((0..2).map(
                            |col| {
                                div().flex_1().min_w_0().overflow_hidden().child(
                                    collection_artwork(
                                        covers.get(row * 2 + col).cloned().flatten(),
                                        false,
                                        cx,
                                    ),
                                )
                            },
                        )));
                }
                collage.into_any_element()
            } else {
                collection_artwork(cover, false, cx)
            }
        } else {
            collection_artwork(cover, artist, cx)
        };
        let focus = self.collection_focus(&key, target, window, cx);
        let keyboard_page = page.clone();
        let label = format!("{title}, {subtitle}");
        div()
            .id(SharedString::from(key))
            .track_focus(&focus)
            .tab_index(0)
            .key_context("CollectionCard")
            .role(gpui_kit::accesskit::Role::Button)
            .aria_label(label)
            .border_1()
            .border_color(rgba(0x00000000))
            .focus(|d| d.border_color(cx.theme().primary))
            .on_action(
                cx.listener(move |this, _: &ActivateCollection, window, cx| {
                    this.navigate(keyboard_page.clone(), window, cx)
                }),
            )
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_1()
            .p_2()
            .rounded_lg()
            .cursor_pointer()
            .hover(|d| d.bg(cx.theme().muted))
            .active(|d| d.bg(cx.theme().border))
            .child(artwork)
            .child(
                div()
                    .mt_2()
                    .truncate()
                    .font_weight(FontWeight::MEDIUM)
                    .when(artist, |d| d.text_center())
                    .child(title),
            )
            .child(
                div()
                    .truncate()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .when(artist, |d| d.text_center())
                    .child(subtitle),
            )
            .on_click(
                cx.listener(move |this, _, window, cx| this.navigate(page.clone(), window, cx)),
            )
            .into_any_element()
    }
}

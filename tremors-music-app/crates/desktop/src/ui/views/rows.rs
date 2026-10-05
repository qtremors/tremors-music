// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Song rows, group cards, context actions, and drag-and-drop.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_row(
        &mut self,
        index: usize,
        _window: &mut Window,
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
            } => div()
                .id(("group", index))
                .w_full()
                .h(px(64.))
                .px_4()
                .flex()
                .items_center()
                .gap_4()
                .cursor_pointer()
                .hover(|s| s.bg(cx.theme().muted))
                .child(cover_element(cover, 44.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(subtitle),
                        ),
                )
                .child(Icon::new(IconName::ChevronRight))
                .on_click(
                    cx.listener(move |this, _, window, cx| this.navigate(page.clone(), window, cx)),
                )
                .into_any_element(),
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
                    .h(px(64.))
                    .px_3()
                    .flex()
                    .items_center()
                    .gap_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .hover(|s| s.bg(cx.theme().muted))
                    .when(playing, |s| s.bg(cx.theme().muted));
                row = row
                    .child(
                        Button::new(("play-track", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Play))
                            .tooltip("Play this track")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                window.focus(&this.focus, cx);
                                if let Some(qindex) = queue_index {
                                    this.queue.select(qindex);
                                } else {
                                    let Some(selected) = song_index(&this.rows, index) else {
                                        return;
                                    };
                                    this.queue.replace(this.visible_songs(), selected);
                                }
                                this.load_current();
                                cx.notify();
                            })),
                    )
                    .when(self.preferences.show_artwork, |row| {
                        row.child(cover_element(song.cover.clone(), 40.))
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(song.title.clone()),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} · {}", song.artist, song.album)),
                            ),
                    )
                    .child(
                        div()
                            .w(px(48.))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(duration_label(song.duration)),
                    )
                    .child(
                        Button::new(("favorite", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Heart))
                            .selected(song.favorite)
                            .tooltip(if song.favorite {
                                "Remove favorite"
                            } else {
                                "Favorite"
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.send_library(LibraryCommand::Favorite(id));
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new(("playlist-add", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::ListPlus))
                            .tooltip("Add to playlist")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.overlay = Some(Overlay::AddToPlaylist(id));
                                cx.notify();
                            })),
                    );
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
                } else if let (Page::Playlist(playlist), Some(entry)) = (self.page.clone(), entry) {
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
                } else {
                    row = row.child(
                        Button::new(("queue-add", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Plus))
                            .tooltip("Add to queue")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.queue.append((*song).clone());
                                this.persist_queue();
                                this.status = "Added to queue.".into();
                                cx.notify();
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
                            "Track details",
                        ] {
                            let view = view.clone();
                            let song = menu_song.clone();
                            menu = menu.item(PopupMenuItem::new(label).on_click(
                                move |_, window, cx| {
                                    let _ = view.update(cx, |this, cx| {
                                        match label {
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
                                            "Add to playlist" => {
                                                this.overlay = Some(Overlay::AddToPlaylist(song.id))
                                            }
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
                                            _ => {
                                                this.overlay = Some(Overlay::Details(Box::new(
                                                    (*song).clone(),
                                                )))
                                            }
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
                        .label("More")
                        .dropdown_menu(build_menu.clone()),
                )
                .context_menu(build_menu)
                .into_any_element()
            }
        }
    }

    pub(in crate::ui) fn render_card(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(Row::Group {
            title,
            subtitle,
            cover,
            page,
        }) = self.rows.get(index).cloned()
        else {
            return div().into_any_element();
        };
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
            if covers.len() > 1 {
                let mut collage = div()
                    .w_full()
                    .h(px(192.))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .rounded_lg();
                for row in 0..2 {
                    collage = collage.child(div().flex().h(px(96.)).children((0..2).map(|col| {
                        div().flex_1().overflow_hidden().child(cover_element(
                            covers
                                .get(row * 2 + col)
                                .cloned()
                                .unwrap_or_else(|| cover.clone()),
                            96.,
                        ))
                    })));
                }
                collage.into_any_element()
            } else {
                cover_element(cover, 192.)
            }
        } else {
            cover_element(cover, 192.)
        };
        div()
            .id(("card", index))
            .flex_1()
            .min_w_0()
            .p_3()
            .rounded_lg()
            .cursor_pointer()
            .hover(|d| d.bg(cx.theme().muted))
            .child(artwork)
            .child(
                div()
                    .mt_3()
                    .truncate()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .child(
                div()
                    .mt_1()
                    .truncate()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(subtitle),
            )
            .on_click(
                cx.listener(move |this, _, window, cx| this.navigate(page.clone(), window, cx)),
            )
            .into_any_element()
    }
}

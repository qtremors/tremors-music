// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Search, sorting, playlist, and queue header actions.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_header(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let mut header = div()
            .p_6()
            .pb_3()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        Button::new("back")
                            .ghost()
                            .icon(Icon::new(IconName::ChevronLeft))
                            .disabled(self.history.is_empty())
                            .on_click(cx.listener(|this, _, window, cx| {
                                if let Some(page) = this.history.pop() {
                                    this.page = page;
                                    this.query.clear();
                                    this.search
                                        .update(cx, |input, cx| input.set_value("", window, cx));
                                    this.rebuild();
                                    cx.notify();
                                }
                            })),
                    )
                    .child(div().flex_1().child(Input::new(&self.search))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex_1()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .child(self.title()),
                    )
                    .when(
                        self.rows.iter().any(|row| matches!(row, Row::Song { .. })),
                        |row| {
                            row.child(
                                Button::new("play-all")
                                    .primary()
                                    .label("Play all")
                                    .icon(Icon::new(IconName::Play))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.queue.replace(this.visible_songs(), 0);
                                        this.load_current();
                                        this.rebuild();
                                        cx.notify();
                                    })),
                            )
                        },
                    )
                    .when(
                        matches!(
                            self.page,
                            Page::Library | Page::Favorites | Page::Artist(_) | Page::Genre(_)
                        ),
                        |row| {
                            let view = cx.entity().downgrade();
                            row.child(
                                Button::new("sort")
                                    .ghost()
                                    .label(format!("Sort: {}", self.preferences.sort.label()))
                                    .dropdown_menu(move |mut menu, _, _| {
                                        for sort in SongSort::ALL {
                                            let view = view.clone();
                                            menu = menu.item(
                                                PopupMenuItem::new(sort.label()).on_click(
                                                    move |_, _, cx| {
                                                        let _ = view.update(cx, |this, cx| {
                                                            this.preferences.sort = sort;
                                                            this.persist_modes();
                                                            this.rebuild();
                                                            cx.notify();
                                                        });
                                                    },
                                                ),
                                            );
                                        }
                                        menu
                                    }),
                            )
                            .child(
                                Button::new("sort-direction")
                                    .ghost()
                                    .label(if self.preferences.descending {
                                        "Descending"
                                    } else {
                                        "Ascending"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.preferences.descending = !this.preferences.descending;
                                        this.persist_modes();
                                        this.rebuild();
                                        cx.notify();
                                    })),
                            )
                        },
                    ),
            );
        if let Page::Playlist(id) = self.page {
            header = header.child(
                div()
                    .flex()
                    .gap_3()
                    .child(div().flex_1().child(Input::new(&self.playlist_name)))
                    .child(Button::new("rename-playlist").label("Rename").on_click(
                        cx.listener(|this, _, window, cx| this.save_playlist(window, cx)),
                    ))
                    .child(
                        Button::new("delete-playlist")
                            .ghost()
                            .label("Delete")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.overlay = Some(Overlay::DeletePlaylist(id));
                                cx.notify();
                            })),
                    ),
            );
        }
        if self.page == Page::Queue {
            header = header.child(
                Button::new("clear-queue")
                    .ghost()
                    .label("Clear queue")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.queue.clear();
                        this.stop();
                        this.rebuild();
                        cx.notify();
                    })),
            );
        }
        header.into_any_element()
    }
}

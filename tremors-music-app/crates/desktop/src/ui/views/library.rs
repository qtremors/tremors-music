// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Empty states and virtualized library lists and galleries.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_content(&mut self, cx: &mut Context<Self>) -> AnyElement {
        if self.page == Page::Folders {
            self.render_folders(cx)
        } else if self.rows.is_empty() {
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .p_8()
                .child(Icon::new(IconName::Music).size_12())
                .child(div().text_lg().child(if self.query.is_empty() {
                    "No tracks here yet"
                } else {
                    "No matching tracks"
                }))
                .child(div().text_color(cx.theme().muted_foreground).child(
                    if self.snapshot.songs.is_empty() {
                        "Add a folder to start building your library."
                    } else {
                        "Choose another view or adjust your search."
                    },
                ))
                .when(self.snapshot.songs.is_empty(), |d| {
                    d.child(
                        Button::new("empty-add-folder")
                            .primary()
                            .label("Add music folder")
                            .disabled(self.picking_folder || self.scanning)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.pick_folder();
                                cx.notify();
                            })),
                    )
                })
                .into_any_element()
        } else if matches!(
            self.page,
            Page::Albums | Page::Artists | Page::Genres | Page::Playlists
        ) {
            let view = cx.entity().downgrade();
            uniform_list(
                "gallery",
                self.rows.len().div_ceil(3),
                move |range, _, cx| {
                    view.update(cx, |this, cx| {
                        range
                            .map(|row| {
                                let mut strip = div().w_full().h(px(280.)).px_4().flex().gap_4();
                                for col in 0..3 {
                                    let index = row * 3 + col;
                                    strip = strip.child(if index < this.rows.len() {
                                        this.render_card(index, cx)
                                    } else {
                                        div().flex_1().into_any_element()
                                    });
                                }
                                strip.into_any_element()
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
                },
            )
            .flex_1()
            .min_h_0()
            .w_full()
            .into_any_element()
        } else {
            let view = cx.entity().downgrade();
            uniform_list("music-list", self.rows.len(), move |range, window, cx| {
                view.update(cx, |this, cx| {
                    range
                        .map(|i| this.render_row(i, window, cx))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
            })
            .flex_1()
            .min_h_0()
            .w_full()
            .into_any_element()
        }
    }
}

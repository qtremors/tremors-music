// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Fullscreen artwork and lyrics composition.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_fullscreen(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if self.immersive {
            let cover_size = (f32::from(window.viewport_size().height) - 438.).clamp(160., 400.);
            let player = self.render_player(cx);
            let cover = self.queue.current().and_then(|s| s.cover.clone());
            let title = self
                .queue
                .current()
                .map(|s| s.title.clone())
                .unwrap_or_else(|| "Nothing playing".into());
            let artist = self
                .queue
                .current()
                .map(|s| s.artist.clone())
                .unwrap_or_default();
            Some(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(player)
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .flex()
                            .p_8()
                            .gap_8()
                            .child(
                                div()
                                    .w(px(420.))
                                    .flex()
                                    .flex_col()
                                    .justify_center()
                                    .items_center()
                                    .gap_5()
                                    .child(cover_element(cover, cover_size, cx))
                                    .child(
                                        div()
                                            .text_3xl()
                                            .text_center()
                                            .font_weight(FontWeight::BOLD)
                                            .child(title),
                                    )
                                    .child(
                                        div()
                                            .text_lg()
                                            .text_center()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(artist),
                                    )
                                    .child(
                                        Button::new("exit-fullscreen")
                                            .ghost()
                                            .label("Back to library")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.fullscreen(window, cx)
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .child(self.render_lyrics(cx)),
                            ),
                    )
                    .child(
                        div()
                            .px_8()
                            .py_2()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(self.status.clone()),
                    )
                    .into_any_element(),
            )
        } else {
            None
        }
    }
}

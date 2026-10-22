// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Plain and synchronized lyrics presentation.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_lyrics(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let text = self
            .queue
            .current()
            .map(|s| s.details.lyrics.clone())
            .unwrap_or_default();
        if text.is_empty() {
            return div().flex_1().flex().flex_col().items_center().justify_center().gap_4().p_8()
                .child(Icon::new(IconName::Mic).size_12().text_color(cx.theme().primary))
                .child(div().text_2xl().font_weight(FontWeight::BOLD).child("Every song has a story"))
                .child(div().max_w(px(400.)).text_center().text_color(cx.theme().muted_foreground)
                    .child("Lyrics appear here when your track contains them. You can also place a matching .lrc file beside your song."))
                .into_any_element();
        }
        let lines = self.lyric_lines.clone();
        if lines.is_empty() {
            return div()
                .id("plain-lyrics")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p_8()
                .text_lg()
                .children(text.lines().map(|line| div().py_2().child(line.to_owned())))
                .into_any_element();
        }
        let active = active_line(&lines, self.position);
        let this = cx.entity().downgrade();
        uniform_list("synced-lyrics", lines.len(), move |range, _, cx| {
            this.update(cx, |_this, cx| {
                range
                    .map(|i| {
                        let line = &lines[i];
                        let seconds = line.seconds;
                        div()
                            .id(("lyric", i))
                            .h(px(72.))
                            .px_8()
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .text_2xl()
                            .text_color(if active == Some(i) {
                                cx.theme().primary
                            } else {
                                cx.theme().muted_foreground
                            })
                            .when(active == Some(i), |d| d.font_weight(FontWeight::BOLD))
                            .child(if line.text.is_empty() {
                                "♪".to_owned()
                            } else {
                                line.text.clone()
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.seek_to(seconds);
                                cx.notify();
                            }))
                            .into_any_element()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
        })
        .track_scroll(&self.lyrics_scroll)
        .flex_1()
        .min_h_0()
        .into_any_element()
    }
}

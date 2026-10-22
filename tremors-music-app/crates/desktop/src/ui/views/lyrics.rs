// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Variable-height lyric rows keep complete lines readable at every width.
use super::*;
impl MusicApp {
    pub(in crate::ui) fn render_lyrics(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let synced = !self.lyric_lines.is_empty();
        if !synced && self.lyric_plain_lines.is_empty() {
            return div().flex_1().flex().flex_col().items_center().justify_center().gap_4().p_8()
                .child(self.render_lyric_tools(cx))
                .child(Icon::new(IconName::Mic).size_12().text_color(cx.theme().primary))
                .child(div().text_2xl().font_weight(FontWeight::BOLD).child("Every song has a story"))
                .child(div().max_w(px(400.)).text_center().text_color(cx.theme().muted_foreground)
                    .child("Lyrics appear here when your track contains them. You can also place a matching .lrc, .ttml or .txt file beside your song."))
                .into_any_element();
        }
        let lines = self.lyric_lines.clone();
        let plain = self.lyric_plain_lines.clone();
        let active = active_line(&lines, self.lyric_position());
        let position = self.lyric_position();
        let offset = self.lyric_offset();
        let this = cx.entity().downgrade();
        let lyrics = list(self.lyrics_scroll.clone(), move |index, _, cx| {
            this.update(cx, |_this, cx| {
                let seconds = lines.get(index).map(|line| line.seconds);
                let text = if synced {
                    lines.get(index).map(|line| line.text.clone())
                } else {
                    plain.get(index).cloned()
                }
                .unwrap_or_default();
                let highlights: Vec<_> = lines
                    .get(index)
                    .into_iter()
                    .flat_map(|line| line.words.iter())
                    .filter(|word| word.seconds <= position)
                    .map(|word| {
                        (
                            word.start..word.end,
                            HighlightStyle {
                                color: Some(cx.theme().primary),
                                ..Default::default()
                            },
                        )
                    })
                    .collect();
                div()
                    .id(("lyric", index))
                    .w_full()
                    .px_8()
                    .py_3()
                    .min_h(px(if synced { 72. } else { 40. }))
                    .when(synced, |d| d.text_2xl().cursor_pointer())
                    .when(!synced, |d| d.text_lg())
                    .text_color(
                        if active == Some(index)
                            && lines.get(index).is_none_or(|line| line.words.is_empty())
                        {
                            cx.theme().primary
                        } else {
                            cx.theme().muted_foreground
                        },
                    )
                    .when(active == Some(index), |d| d.font_weight(FontWeight::BOLD))
                    .child(
                        StyledText::new(if text.is_empty() {
                            if synced {
                                "♪".to_owned()
                            } else {
                                " ".to_owned()
                            }
                        } else {
                            text
                        })
                        .with_highlights(highlights),
                    )
                    .when_some(seconds, |d, seconds| {
                        d.on_click(cx.listener(move |this, _, _, cx| {
                            this.seek_to(seconds + offset);
                            cx.notify();
                        }))
                    })
                    .into_any_element()
            })
            .unwrap_or_else(|_| div().into_any_element())
        })
        .flex_1()
        .min_h_0();
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(self.render_lyric_tools(cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(lyrics)
                    .child(Scrollbar::vertical(&self.lyrics_scroll)),
            )
            .into_any_element()
    }
}

// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Persistent transport, progress, volume, and player shortcuts.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_player(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let current = self.queue.current();
        let cover = current.and_then(|s| s.cover.clone());
        let title = current
            .map(|s| s.title.clone())
            .unwrap_or_else(|| "Nothing playing".into());
        let artist = current
            .map(|s| s.artist.clone())
            .unwrap_or_else(|| "Choose a song from your library".into());
        let disabled = current.is_none();
        div()
            .flex_shrink_0()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .p_4()
            .flex()
            .items_center()
            .gap_5()
            .child(
                div()
                    .w(px(280.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(cover_element(cover, 52.))
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
                                    .child(artist),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .justify_center()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("shuffle")
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(IconName::Shuffle))
                                    .selected(self.queue.shuffle)
                                    .tooltip("Shuffle")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.queue.set_shuffle(!this.queue.shuffle);
                                        this.persist_modes();
                                        this.rebuild();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("previous")
                                    .ghost()
                                    .icon(Icon::new(IconName::SkipBack))
                                    .disabled(disabled)
                                    .tooltip("Previous / restart")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.previous();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("play-pause")
                                    .primary()
                                    .icon(Icon::new(if self.is_playing {
                                        IconName::Pause
                                    } else {
                                        IconName::Play
                                    }))
                                    .disabled(self.loading)
                                    .tooltip(if self.loading {
                                        "Loading audio…"
                                    } else {
                                        "Play / pause (Space)"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle();
                                        this.rebuild();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("next")
                                    .ghost()
                                    .icon(Icon::new(IconName::SkipForward))
                                    .disabled(disabled)
                                    .tooltip("Next")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.next();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("repeat")
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(if self.queue.repeat == RepeatMode::One {
                                        IconName::Repeat1
                                    } else {
                                        IconName::Repeat
                                    }))
                                    .selected(self.queue.repeat != RepeatMode::Off)
                                    .tooltip(format!("Repeat: {:?}", self.queue.repeat))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.queue.repeat = this.queue.repeat.cycle();
                                        this.persist_modes();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(48.))
                                    .text_xs()
                                    .child(duration_label(self.position)),
                            )
                            .child(div().flex_1().child(Slider::new(&self.seek).horizontal()))
                            .child(
                                div()
                                    .w(px(48.))
                                    .text_xs()
                                    .child(duration_label(self.duration)),
                            ),
                    ),
            )
            .child(
                div()
                    .w(px(250.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        Button::new("mute")
                            .ghost()
                            .small()
                            .icon(Icon::new(if self.muted {
                                IconName::VolumeX
                            } else {
                                IconName::Volume2
                            }))
                            .tooltip("Mute · Ctrl+M")
                            .on_click(cx.listener(|this, _, _, cx| this.mute(cx))),
                    )
                    .child(div().flex_1().child(Slider::new(&self.volume).horizontal()))
                    .child(
                        Button::new("show-lyrics")
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Mic))
                            .tooltip("Lyrics · Ctrl+L")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate(Page::Lyrics, window, cx)
                            })),
                    )
                    .child(
                        Button::new("fullscreen-player")
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Maximize))
                            .tooltip("Fullscreen player · F11")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.fullscreen(window, cx)),
                            ),
                    )
                    .child(
                        Button::new("show-queue")
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::ListMusic))
                            .tooltip("Queue")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate(Page::Queue, window, cx)
                            })),
                    ),
            )
            .into_any_element()
    }
}

// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Persistent desktop transport, now playing, and local playback shortcuts.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_player(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let current = self.queue.current().cloned();
        let disabled = current.is_none() || self.metadata_busy();
        let title = current
            .as_ref()
            .map(|s| s.title.clone())
            .unwrap_or_else(|| "Tremors Music".into());
        let artist = current
            .as_ref()
            .map(|s| s.artist.clone())
            .unwrap_or_else(|| "Your music. Your space.".into());
        let mut transport = div()
            .w(px(184.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .gap_1();
        transport = transport
            .child(
                Button::new("shuffle")
                    .ghost()
                    .small()
                    .icon(
                        Icon::new(IconName::Shuffle).when(self.queue.shuffle, |icon| {
                            icon.text_color(cx.theme().primary)
                        }),
                    )
                    .selected(self.queue.shuffle)
                    .toggled(self.queue.shuffle)
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
                    .small()
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
                    .ghost()
                    .icon(
                        Icon::new(if self.is_playing {
                            IconName::Pause
                        } else {
                            IconName::Play
                        })
                        .size_6(),
                    )
                    .disabled(
                        self.loading
                            || self.metadata_busy()
                            || (disabled
                                && !self.rows.iter().any(|row| matches!(row, Row::Song { .. }))),
                    )
                    .tooltip(if self.loading {
                        "Loading audio…"
                    } else {
                        "Play / pause · Space"
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
                    .small()
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
                    .toggled(self.queue.repeat != RepeatMode::Off)
                    .tooltip(format!("Repeat: {:?}", self.queue.repeat))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.queue.repeat = this.queue.repeat.cycle();
                        this.persist_modes();
                        cx.notify();
                    })),
            );
        let mut now_playing = div()
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(cx.theme().muted)
            .child(
                div()
                    .id("player-artwork")
                    .cursor_pointer()
                    .child(motion::enter(
                        div().child(cover_element(
                            current.as_ref().and_then(|s| s.cover.clone()),
                            44.,
                            cx,
                        )),
                        SharedString::from(format!(
                            "player-cover-{}-{}",
                            current.as_ref().map_or(0, |s| s.id),
                            current.as_ref().map_or(0, |s| s.details.modified_ns)
                        )),
                        self.preferences.reduce_motion,
                    ))
                    .on_click(cx.listener(|this, _, window, cx| this.fullscreen(window, cx))),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .truncate()
                            .text_center()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(title),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_center()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(artist),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(32.))
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(duration_label(self.position)),
                            )
                            .child(
                                div().flex_1().child(
                                    Slider::new(&self.seek)
                                        .horizontal()
                                        .disabled(disabled || self.duration <= 0.),
                                ),
                            )
                            .child(
                                div()
                                    .w(px(32.))
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(duration_label(self.duration)),
                            ),
                    ),
            );
        if let Some(song) = current {
            now_playing = now_playing.child(
                Button::new("player-favorite")
                    .ghost()
                    .small()
                    .icon(
                        Icon::new(IconName::Heart)
                            .when(song.favorite, |icon| icon.text_color(cx.theme().primary)),
                    )
                    .selected(song.favorite)
                    .toggled(song.favorite)
                    .tooltip("Favorite")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.send_library(LibraryCommand::Favorite(song.id));
                        cx.notify();
                    })),
            );
        }
        let tools = div()
            .w(px(226.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_1()
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
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Slider::new(&self.volume).horizontal()),
            )
            .child(
                Button::new("sleep-timer")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Clock))
                    .selected(self.sleep_timer.is_some())
                    .tooltip(if self.timer_label.is_empty() {
                        "Sleep timer".into()
                    } else {
                        format!("Sleep timer · {}", self.timer_label)
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.overlay = Some(Overlay::SleepTimer);
                        cx.notify();
                    })),
            )
            .child(
                Button::new("show-lyrics")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Mic))
                    .selected(self.page == Page::Lyrics)
                    .tooltip("Lyrics · Ctrl+L")
                    .on_click(
                        cx.listener(|this, _, window, cx| this.navigate(Page::Lyrics, window, cx)),
                    ),
            )
            .child(
                Button::new("show-queue")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::ListMusic))
                    .selected(self.page == Page::Queue)
                    .tooltip("Up next · Ctrl+Q")
                    .on_click(
                        cx.listener(|this, _, window, cx| this.navigate(Page::Queue, window, cx)),
                    ),
            )
            .child(
                Button::new("fullscreen-player")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Maximize))
                    .tooltip("Fullscreen · F11")
                    .on_click(cx.listener(|this, _, window, cx| this.fullscreen(window, cx))),
            );
        div()
            .h(px(84.))
            .flex_shrink_0()
            .px_4()
            .flex()
            .items_center()
            .gap_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .child(transport)
            .child(now_playing)
            .child(tools)
            .into_any_element()
    }
}

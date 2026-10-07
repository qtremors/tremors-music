// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Appearance, playback preferences, and reset controls.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_settings(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div()
            .id("settings-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p_6()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Appearance"),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        Button::new("theme-dark")
                            .label("Dark")
                            .selected(!self.preferences.light_theme)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.preferences.light_theme = false;
                                this.apply_theme(window, cx);
                                this.persist_modes();
                            })),
                    )
                    .child(
                        Button::new("theme-light")
                            .label("Light")
                            .selected(self.preferences.light_theme)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.preferences.light_theme = true;
                                this.apply_theme(window, cx);
                                this.persist_modes();
                            })),
                    ),
            );
        let mut accents = div().flex().flex_wrap().gap_2();
        for name in [
            "Neutral", "Red", "Blue", "Purple", "Orange", "Teal", "Green",
        ] {
            accents = accents.child(
                Button::new(name)
                    .label(name)
                    .selected(self.preferences.accent == name)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.preferences.accent = name.into();
                        this.apply_theme(window, cx);
                        this.persist_modes();
                    })),
            );
        }
        body=body.child(div().child("Accent color").child(accents))
            .child(Switch::new("artwork-toggle").label("Show artwork in song lists").checked(self.preferences.show_artwork).on_click(cx.listener(|this,checked:&bool,_,cx| {
                this.preferences.show_artwork= *checked;this.persist_modes();cx.notify();
            })))
            .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child("Playback & library"))
            .child(Switch::new("watch-toggle").label("Automatically update music folders").checked(self.preferences.watch_folders).on_click(cx.listener(|this,checked:&bool,_,cx| {
                this.preferences.watch_folders= *checked;this.persist_modes();cx.notify();
            })))
            .child(Switch::new("replaygain-toggle").label("Normalize tagged tracks with ReplayGain").checked(self.preferences.replay_gain).on_click(cx.listener(|this,checked:&bool,_,cx| {
                this.preferences.replay_gain= *checked;this.persist_modes();
                this.status="ReplayGain setting applies when the next track starts.".into();cx.notify();
            })))
            .child(div().text_color(cx.theme().muted_foreground).child("Queue and playback position are saved. Playback stays paused when you reopen the app. ReplayGain uses embedded track gain and peak tags."))
            .child(Button::new("clean-covers").label("Clean unused artwork").disabled(self.scanning).on_click(cx.listener(|this,_,_,cx| {
                this.send_library(LibraryCommand::CleanArtwork);this.status="Unused artwork cleanup requested.".into();cx.notify();
            })))
            .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child("Keyboard shortcuts"))
            .child("Space: play/pause · Left/Right: previous/next · Shift+Left/Right: seek 10 seconds")
            .child("Up/Down: volume · Ctrl+M: mute · Ctrl+K or /: search · Ctrl+Q: queue")
            .child("Ctrl+L: lyrics · Ctrl+F or F11: fullscreen player · Alt+Left: back · Escape: close")
            .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child("Reset library"))
            .child(div().text_color(cx.theme().muted_foreground).child("Reset removes library records, playlists, favorites and play counts. Your music files are never deleted."))
            .child(div().flex().gap_3()
                .child(Button::new("reset-keep").danger().label("Reset, keep folders").disabled(self.scanning).on_click(cx.listener(|this,_,_,cx| {this.overlay=Some(Overlay::Reset(false));cx.notify();})))
                .child(Button::new("reset-all").danger().label("Reset everything").disabled(self.scanning).on_click(cx.listener(|this,_,_,cx| {this.overlay=Some(Overlay::Reset(true));cx.notify();}))));
        body.into_any_element()
    }
}

// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Appearance and local playback preferences.

use super::*;

fn settings_section(title: &str, cx: &App) -> Div {
    div()
        .p_5()
        .rounded_lg()
        .bg(cx.theme().sidebar)
        .border_1()
        .border_color(cx.theme().border)
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.to_owned()),
        )
}

impl MusicApp {
    pub(in crate::ui) fn render_settings(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let mut appearance = settings_section("Appearance", cx)
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Choose the look that feels like you."),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        Button::new("theme-dark")
                            .label("Dark")
                            .selected(!self.preferences.light_theme && !self.preferences.oled_theme)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.preferences.light_theme = false;
                                this.preferences.oled_theme = false;
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
                    )
                    .child(
                        Button::new("theme-oled")
                            .label("OLED")
                            .selected(!self.preferences.light_theme && self.preferences.oled_theme)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.preferences.light_theme = false;
                                this.preferences.oled_theme = true;
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
        appearance = appearance
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Accent color"),
            )
            .child(accents)
            .child(
                Switch::new("artwork-toggle")
                    .label("Show artwork in song lists")
                    .checked(self.preferences.show_artwork)
                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                        this.preferences.show_artwork = *checked;
                        this.persist_modes();
                        cx.notify();
                    })),
            );
        appearance = appearance.child(
            Switch::new("reduce-motion")
                .label("Reduce motion")
                .checked(self.preferences.reduce_motion)
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    this.preferences.reduce_motion = *checked;
                    this.persist_modes();
                    cx.notify();
                })),
        );
        let playback = settings_section("Playback & library", cx)
            .child(Switch::new("watch-toggle").label("Automatically update music folders").checked(self.preferences.watch_folders)
                .on_click(cx.listener(|this, checked: &bool, _, cx| { this.preferences.watch_folders = *checked; this.persist_modes(); cx.notify(); })))
            .child(Switch::new("replaygain-toggle").label("Normalize tagged tracks with ReplayGain").checked(self.preferences.replay_gain)
                .on_click(cx.listener(|this, checked: &bool, _, cx| { this.preferences.replay_gain = *checked; this.persist_modes(); this.status = "ReplayGain setting applies when the next track starts.".into(); cx.notify(); })))
            .child(div().text_sm().text_color(cx.theme().muted_foreground).child("Your queue and position are saved automatically. Playback stays paused when you reopen the app. ReplayGain uses the gain and peak tags in your files."))
            .child(Button::new("clean-covers").label("Clean unused artwork").disabled(self.scanning)
                .on_click(cx.listener(|this, _, _, cx| { this.send_library(LibraryCommand::CleanArtwork); this.status = "Unused artwork cleanup requested.".into(); cx.notify(); })));
        let mut shortcuts = settings_section("Keyboard shortcuts", cx);
        for (keys, action) in [
            ("Space", "Play / pause"),
            ("Left / Right", "Previous / next track"),
            ("Shift + Left / Right", "Seek 10 seconds"),
            ("Up / Down", "Adjust volume"),
            ("Ctrl + M", "Mute / unmute"),
            ("Ctrl + K or /", "Search your library"),
            ("Ctrl + Q", "Up next"),
            ("Ctrl + L", "Lyrics"),
            ("Ctrl + F or F11", "Fullscreen player"),
            ("Alt + Left", "Go back"),
            ("Page Up / Page Down", "Scroll the current page"),
            ("Home / End", "Go to the start / end"),
            ("Enter / Space", "Open a focused collection"),
            ("Escape", "Close dialog / leave fullscreen"),
        ] {
            shortcuts = shortcuts.child(
                div()
                    .flex()
                    .gap_4()
                    .text_sm()
                    .child(
                        div()
                            .w(px(180.))
                            .text_color(cx.theme().muted_foreground)
                            .child(keys),
                    )
                    .child(action),
            );
        }
        let backup = settings_section("Backup & restore",cx)
            .child(div().text_sm().text_color(cx.theme().muted_foreground).child("Save playlists, favorites, play history, preferences and custom images in a local curation backup. Music files are kept separately."))
            .child(div().flex().gap_3()
                .child(Button::new("backup-curation").label("Save backup").disabled(self.picking_file || self.scanning).on_click(cx.listener(|this,_,_,cx| {this.pick_file(super::super::file_tasks::FileTask::Backup);cx.notify();})))
                .child(Button::new("restore-curation").label("Review backup…").disabled(self.picking_file || self.scanning).on_click(cx.listener(|this,_,_,cx| {this.pick_file(super::super::file_tasks::FileTask::RestoreBackup);cx.notify();}))));
        let reset = settings_section("Library data", cx)
            .child(div().text_sm().text_color(cx.theme().muted_foreground).child("Reset removes library records, playlists, favorites, and play counts. Your music files are never deleted."))
            .child(div().flex().gap_3()
                .child(Button::new("reset-keep").danger().label("Reset, keep folders").disabled(self.scanning)
                    .on_click(cx.listener(|this, _, window, cx| { this.open_overlay(Overlay::Reset(false), window, cx); cx.notify(); })))
                .child(Button::new("reset-all").danger().label("Reset everything").disabled(self.scanning)
                    .on_click(cx.listener(|this, _, window, cx| { this.open_overlay(Overlay::Reset(true), window, cx); cx.notify(); }))));
        div()
            .id("settings-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px_8()
            .pb_8()
            .flex()
            .flex_col()
            .gap_5()
            .child(appearance)
            .child(playback)
            .child(self.render_sound_settings(cx))
            .child(backup)
            .child(shortcuts)
            .child(reset)
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "Tremors Music · v{} · Your music stays on this device",
                        env!("CARGO_PKG_VERSION")
                    )),
            )
            .track_scroll(&self.page_scroll)
            .vertical_scrollbar(&self.page_scroll)
            .into_any_element()
    }
}

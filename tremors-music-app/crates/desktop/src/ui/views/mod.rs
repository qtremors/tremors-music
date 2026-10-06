// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Desktop composition. Feature views live in sibling modules.

mod folders;
mod fullscreen;
mod header;
mod library;
mod lyrics;
mod overlays;
mod player;
mod rows;
mod sidebar;

use super::*;

impl Render for MusicApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = self.render_header(cx);
        let content = self.render_content(cx);
        let sidebar = self.render_sidebar(window, cx);
        let player = self.render_player(cx);
        let overlay = self.render_overlay(cx);
        let immersive = self.render_fullscreen(cx);

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_sm()
            .track_focus(&self.focus)
            .key_context("Player")
            .on_action(cx.listener(|this, _: &TogglePlayback, _, cx| {
                this.toggle();
                this.rebuild();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NextTrack, _, cx| {
                this.next();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &PreviousTrack, _, cx| {
                this.previous();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &VolumeUp, window, cx| {
                this.set_volume(this.preferences.volume + 0.05, window, cx)
            }))
            .on_action(cx.listener(|this, _: &VolumeDown, window, cx| {
                this.set_volume(this.preferences.volume - 0.05, window, cx)
            }))
            .on_action(cx.listener(|this, _: &Mute, _, cx| this.mute(cx)))
            .on_action(cx.listener(|this, _: &SeekForward, _, cx| {
                this.seek_to(this.position + 10.);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &SeekBackward, _, cx| {
                this.seek_to(this.position - 10.);
                cx.notify();
            }))
            .on_action(
                cx.listener(|this, _: &ShowQueue, window, cx| {
                    this.navigate(Page::Queue, window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &ShowLyrics, window, cx| {
                this.navigate(Page::Lyrics, window, cx)
            }))
            .on_action(cx.listener(|this, _: &Fullscreen, window, cx| this.fullscreen(window, cx)))
            .on_action(cx.listener(|this, _: &Back, window, cx| {
                if let Some(page) = this.history.pop() {
                    this.page = page;
                    this.query.clear();
                    this.search
                        .update(cx, |input, cx| input.set_value("", window, cx));
                    this.rebuild();
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.navigate(Page::Search, window, cx);
                this.search.update(cx, |input, cx| input.focus(window, cx));
            }))
            .on_action(cx.listener(|this, _: &Escape, window, cx| {
                this.overlay = None;
                if this.immersive {
                    this.fullscreen(window, cx);
                }
                window.focus(&this.focus, cx);
                cx.notify();
            }))
            .when(!self.immersive, |root| {
                root.child(
                    div().flex_1().min_h_0().flex().child(sidebar).child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(header)
                            .child(content)
                            .child(
                                div()
                                    .px_6()
                                    .py_2()
                                    .border_t_1()
                                    .border_color(cx.theme().border)
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(self.status.clone()),
                                    )
                                    .when(self.scanning, |d| {
                                        d.child(
                                            Button::new("status-cancel-scan")
                                                .ghost()
                                                .small()
                                                .label("Cancel")
                                                .on_click(cx.listener(|this, _, _, _| {
                                                    this.library
                                                        .cancel
                                                        .store(true, Ordering::Relaxed)
                                                })),
                                        )
                                    })
                                    .when(!self.warnings.is_empty(), |d| {
                                        d.child(
                                            Button::new("show-warnings")
                                                .ghost()
                                                .small()
                                                .label("View warnings")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.navigate(Page::Folders, window, cx)
                                                })),
                                        )
                                    }),
                            ),
                    ),
                )
            })
            .children(immersive)
            .child(player)
            .children(overlay)
    }
}

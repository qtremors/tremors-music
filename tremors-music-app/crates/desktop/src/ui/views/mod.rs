// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

mod player;
use super::*;

impl Render for MusicApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tracks = self.visible_songs();
        let mut list = div().id("library").flex_1().overflow_y_scroll();
        for (index, song) in tracks.iter().enumerate() {
            list = list.child(Button::new(("track", index)).ghost().label(song.title.clone()).on_click(cx.listener(move |this, _, _, cx| {
                this.queue.replace(this.visible_songs(), index);
                this.load_current();
                cx.notify();
            })));
        }
        div().size_full().flex().flex_col().bg(cx.theme().background).text_color(cx.theme().foreground)
            .track_focus(&self.focus).key_context("Player")
            .child(div().p_4().flex().gap_3().child("Tremors Music")
                .child(Button::new("add-folder").label("Add music folder").disabled(self.picking_folder || self.scanning).on_click(cx.listener(|this, _, _, cx| { this.pick_folder(); cx.notify(); })))
                .child(Button::new("rescan").label("Scan folders").disabled(self.scanning).on_click(cx.listener(|this, _, _, cx| { this.start_scan(); cx.notify(); }))))
            .child(list).child(self.status.clone()).child(self.render_player(cx))
    }
}

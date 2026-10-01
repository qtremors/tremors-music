// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use super::*;
impl MusicApp {
    pub(in crate::ui) fn render_player(&mut self, cx: &mut Context<Self>) -> AnyElement {
        div().p_4().flex().gap_3()
            .child(Button::new("previous").label("Previous").on_click(cx.listener(|this, _, _, cx| { this.previous(); cx.notify(); })))
            .child(Button::new("toggle").label(if self.is_playing { "Pause" } else { "Play" }).on_click(cx.listener(|this, _, _, cx| { this.toggle(); cx.notify(); })))
            .child(Button::new("next").label("Next").on_click(cx.listener(|this, _, _, cx| { this.next(); cx.notify(); })))
            .into_any_element()
    }
}

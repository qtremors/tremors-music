// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Shared input geometry. Keep the content line taller than its font metrics and
//! prevent flex layouts from squeezing the editor below its intended height.
use super::*;
use gpui_kit::component::input::{Textarea, TextareaState};

pub(super) fn field(state: &Entity<InputState>) -> Input {
    Input::new(state)
        .small()
        .min_h(px(38.))
        .max_h(px(38.))
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .px(px(12.))
        .py(px(7.))
        .text_size(px(14.))
        .line_height(px(22.))
        .rounded_md()
}

pub(super) fn search(state: &Entity<InputState>) -> Input {
    field(state)
        .min_h(px(32.))
        .max_h(px(32.))
        .py(px(4.))
        .px(px(10.))
}

pub(super) fn paragraph(state: &Entity<TextareaState>, height: f32) -> Textarea {
    Textarea::new(state)
        .large()
        .h(px(height))
        .min_h(px(height))
        .max_h(px(height))
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .p_0()
        .text_size(px(14.))
        .line_height(px(22.))
        .rounded_md()
}

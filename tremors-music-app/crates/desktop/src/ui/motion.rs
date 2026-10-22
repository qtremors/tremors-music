// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Short, finite entry transitions share the persisted reduced-motion preference.
use super::*;
pub(super) fn enter(element: Div, key: impl Into<ElementId>, reduced: bool) -> AnyElement {
    if reduced {
        element.into_any_element()
    } else {
        element
            .with_animation(
                key,
                Animation::new(Duration::from_millis(180)),
                |element, delta| {
                    element
                        .relative()
                        .top(px(4. * (1. - delta)))
                        .opacity(0.7 + 0.3 * delta)
                },
            )
            .into_any_element()
    }
}

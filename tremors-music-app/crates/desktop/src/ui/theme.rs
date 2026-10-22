// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Translate persisted appearance preferences into the application theme.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn apply_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        Theme::change(
            if self.preferences.light_theme {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            },
            Some(window),
            cx,
        );
        let color = match self.preferences.accent.as_str() {
            "Red" => 0xe5484d,
            "Blue" => 0x3b82f6,
            "Purple" => 0xa855f7,
            "Orange" => 0xf97316,
            "Teal" => 0x14b8a6,
            "Green" => 0x22c55e,
            _ => {
                if self.preferences.light_theme {
                    0x202024
                } else {
                    0xf4f4f5
                }
            }
        };
        let foreground = if (self.preferences.accent == "Neutral" && !self.preferences.light_theme)
            || matches!(
                self.preferences.accent.as_str(),
                "Orange" | "Green" | "Teal"
            ) {
            rgb(0x18181b).into()
        } else {
            rgb(0xffffff).into()
        };
        Theme::update(cx, |theme| {
            let (background, sidebar, muted, border, text, secondary) =
                if self.preferences.light_theme {
                    (0xffffff, 0xf5f5f7, 0xededf0, 0xe5e5e8, 0x202024, 0x737378)
                } else if self.preferences.oled_theme {
                    (0x000000, 0x090909, 0x181818, 0x242424, 0xf5f5f7, 0xa1a1a6)
                } else {
                    (0x1c1c1e, 0x252527, 0x303033, 0x363639, 0xf5f5f7, 0xa1a1a6)
                };
            theme.colors.background = rgb(background).into();
            theme.colors.sidebar = rgb(sidebar).into();
            theme.colors.muted = rgb(muted).into();
            theme.colors.border = rgb(border).into();
            theme.colors.foreground = rgb(text).into();
            theme.colors.muted_foreground = rgb(secondary).into();
            let primary: Hsla = rgb(color).into();
            let mut hover = primary;
            hover.l = (hover.l + 0.06).min(1.);
            let mut active = primary;
            active.l = (active.l - 0.06).max(0.);
            theme.colors.primary = primary;
            theme.colors.primary_foreground = foreground;
            theme.colors.primary_hover = hover;
            theme.colors.primary_active = active;
            theme.colors.button_primary = rgb(color).into();
            theme.colors.button_primary_foreground = foreground;
            theme.colors.button_primary_hover = hover;
            theme.colors.button_primary_active = active;
            theme.colors.ring = rgb(color).into();
            theme.colors.slider_bar = rgb(color).into();
        });
        cx.notify();
    }
}

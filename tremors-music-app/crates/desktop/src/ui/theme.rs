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
            theme.colors.primary = rgb(color).into();
            theme.colors.primary_foreground = foreground;
            theme.colors.primary_hover = rgb(color).into();
            theme.colors.primary_active = rgb(color).into();
            theme.colors.button_primary = rgb(color).into();
            theme.colors.button_primary_foreground = foreground;
            theme.colors.button_primary_hover = rgb(color).into();
            theme.colors.button_primary_active = rgb(color).into();
            theme.colors.ring = rgb(color).into();
            theme.colors.slider_bar = rgb(color).into();
        });
        cx.notify();
    }
}

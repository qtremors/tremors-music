// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Modal focus ownership is independent of the content of each dialog.
use super::*;
impl MusicApp {
    pub(in crate::ui) fn open_overlay(
        &mut self,
        overlay: Overlay,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.overlay.is_none() {
            self.overlay_return_focus = window
                .focused(cx)
                .filter(|focus| self.focus.contains(focus, window));
        }
        self.overlay_scroll = ScrollHandle::new();
        match &overlay {
            Overlay::CreatePlaylist | Overlay::RenamePlaylist(_) => self
                .playlist_name
                .update(cx, |input, cx| input.focus(window, cx)),
            Overlay::RelocateFolder(_) => self
                .folder_path
                .update(cx, |input, cx| input.focus(window, cx)),
            _ => window.focus(&self.overlay_focus, cx),
        }
        self.overlay = Some(overlay);
        cx.notify();
    }
    pub(in crate::ui) fn close_overlay(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(overlay) = self.overlay.take() {
            if matches!(overlay, Overlay::RestoreBackup) {
                self.backup_review = None;
            }
            let focus = self
                .overlay_return_focus
                .take()
                .unwrap_or_else(|| self.focus.clone());
            let focus = if window.is_action_available_in(&Escape, &focus) {
                focus
            } else {
                self.focus.clone()
            };
            window.focus(&focus, cx);
            cx.notify();
        }
    }
}

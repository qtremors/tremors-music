// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Restore review and path mapping. Parsing and transactional merging belong to core.
use super::*;
pub(super) struct BackupReview {
    document: tremors_core::backup::Backup,
    from: Entity<InputState>,
    to: Entity<InputState>,
    preferences: bool,
    _subscriptions: Vec<Subscription>,
    matching: usize,
    matching_key: Option<(String, String, usize)>,
}
impl MusicApp {
    pub(in crate::ui) fn review_backup(
        &mut self,
        document: tremors_core::backup::Backup,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let from = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Original folder (optional)")
                .default_value(
                    document
                        .roots
                        .first()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                )
        });
        let to = cx.new(|cx| InputState::new(window, cx).placeholder("New folder (optional)"));
        let subscriptions = [&from, &to]
            .into_iter()
            .map(|input| cx.subscribe(input, |_, _, _: &InputEvent, cx| cx.notify()))
            .collect();
        self.backup_review = Some(BackupReview {
            document,
            from,
            to,
            preferences: false,
            _subscriptions: subscriptions,
            matching: 0,
            matching_key: None,
        });
        self.open_overlay(Overlay::RestoreBackup, window, cx);
    }
    fn restore_request(&mut self, cx: &App) -> Option<tremors_core::backup::Restore> {
        let review = self.backup_review.take()?;
        let from = review.from.read(cx).value().trim().to_owned();
        let to = review.to.read(cx).value().trim().to_owned();
        Some(tremors_core::backup::Restore {
            backup: review.document,
            remap: (!from.is_empty() && !to.is_empty())
                .then(|| (PathBuf::from(from), PathBuf::from(to))),
            preferences: review.preferences,
        })
    }
    pub(in crate::ui) fn render_backup_review(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let Some(review) = &mut self.backup_review else {
            return div().into_any_element();
        };
        let from = review.from.read(cx).value().trim().to_owned();
        let to = review.to.read(cx).value().trim().to_owned();
        let key = (
            from.clone(),
            to.clone(),
            Arc::as_ptr(&self.snapshot) as usize,
        );
        if review.matching_key.as_ref() != Some(&key) {
            let mapping = (!from.is_empty() && !to.is_empty())
                .then(|| (PathBuf::from(from), PathBuf::from(to)));
            let paths: std::collections::HashSet<_> =
                self.snapshot.songs.iter().map(|s| &s.path).collect();
            review.matching = review
                .document
                .songs
                .iter()
                .filter(|s| {
                    let path = mapping
                        .as_ref()
                        .and_then(|(old, new)| s.path.strip_prefix(old).ok().map(|p| new.join(p)))
                        .unwrap_or_else(|| s.path.clone());
                    paths.contains(&path)
                })
                .count();
            review.matching_key = Some(key);
        }
        let matching = review.matching;
        let preferences = Switch::new("restore-preferences")
            .label("Also restore preferences")
            .checked(review.preferences)
            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                if let Some(review) = &mut this.backup_review {
                    review.preferences = *checked;
                }
                cx.notify();
            }));
        let restore = Button::new("apply-restore")
            .primary()
            .label("Restore curation")
            .disabled(self.scanning)
            .on_click(cx.listener(|this, _, window, cx| {
                if let Some(restore) = this.restore_request(cx) {
                    this.send_library(LibraryCommand::RestoreBackup(Box::new(restore)));
                    this.status = "Restoring library curation…".into();
                    this.close_overlay(window, cx);
                    cx.notify();
                }
            }));
        div().flex().flex_col().gap_3()
            .child(div().text_xl().font_weight(FontWeight::BOLD).child("Restore library curation"))
            .child(format!("{} playlists · {} track records · {} custom images",
                review.document.playlists.len(), review.document.songs.len(), review.document.artwork.len()))
            .child("Scan your music first. Restore adds new playlists and merges favorites and play history; it keeps existing playlists and music files.")
            .child(div().text_sm().child("If your music moved, map one original folder to its new location:"))
            .child(inputs::field(&review.from)).child(inputs::field(&review.to))
            .child(format!("{} of {} visible tracks match. Missing tracks are skipped and reported.",
                matching, review.document.songs.len()))
            .child(preferences).child(restore).into_any_element()
    }
}

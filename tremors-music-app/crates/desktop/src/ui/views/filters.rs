// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Non-destructive library visibility and folder exclusions.
use super::*;
impl MusicApp {
    pub(in crate::ui) fn render_filters(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut filters = div().p_4().rounded_lg().border_1().border_color(cx.theme().border).flex().flex_col().gap_3()
            .child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child("Library filters"))
            .child(div().text_sm().text_color(cx.theme().muted_foreground).child("Hide short tracks and excluded folders without losing favorites, playlists, or play counts. Use 0 to include every duration."))
            .child(div().flex().items_center().gap_3()
                .child(div().w(px(170.)).child(Input::new(&self.minimum_duration)))
                .child(Button::new("apply-minimum-duration").label("Apply duration filter").on_click(cx.listener(|this,_,_,cx| {
                    match this.minimum_duration.read(cx).value().trim().parse::<u32>() {
                        Ok(seconds) if seconds <= 86400 => { this.preferences.minimum_duration = seconds; this.persist_modes(); this.status = "Library filter updated.".into(); },
                        _ => this.status = "Minimum duration must be a whole number from 0 to 86400 seconds.".into(),
                    } cx.notify();
                }))))
            .child(Button::new("exclude-folder").label("Exclude a folder").disabled(self.picking_file || self.scanning).on_click(cx.listener(|this,_,_,cx| {
                this.pick_file(crate::ui::file_tasks::FileTask::ExcludeFolder); cx.notify();
            })));
        for (index, path) in self.preferences.excluded_folders.iter().enumerate() {
            let path = path.clone();
            filters = filters.child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().flex_1().min_w_0().child(path.display().to_string()))
                    .child(
                        Button::new(("include-folder", index))
                            .ghost()
                            .label("Include again")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.preferences.excluded_folders.retain(|p| p != &path);
                                this.persist_modes();
                                this.status =
                                    "Folder included. Scan to discover any new tracks.".into();
                                cx.notify();
                            })),
                    ),
            );
        }
        filters.into_any_element()
    }
}

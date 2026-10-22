// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Music-folder management and scan diagnostics.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_folders(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let mut body=div().id("folders-content").flex_1().min_h_0().overflow_y_scroll().flex().flex_col().px_8().pb_8().gap_4()
            .child(div().text_color(cx.theme().muted_foreground).child("Choose folders containing MP3, FLAC, WAV, Ogg Vorbis, or AAC/ALAC in M4A. Scanning never changes your music files."))
            .child(div().flex().gap_3()
                .child(Button::new("folder-add").primary().label(if self.picking_folder {"Choosing folder…"} else {"Add music folder"})
                    .disabled(self.scanning || self.picking_folder).icon(Icon::new(IconName::FolderPlus)).on_click(cx.listener(|this,_,_,cx| {this.pick_folder();cx.notify();})))
                .child(Button::new("scan").label(if self.scanning {"Scanning…"} else {"Scan folders"}).disabled(self.scanning || self.snapshot.roots.is_empty())
                    .on_click(cx.listener(|this,_,_,cx| {this.start_scan();cx.notify();})))
                .when(self.scanning,|row|row.child(Button::new("cancel-scan").label("Cancel scan").on_click(cx.listener(|this,_,_,_| {this.library.cancel.store(true,Ordering::Relaxed);}))))) ;
        body = body.child(
            div()
                .flex()
                .gap_3()
                .child(div().flex_1().child(Input::new(&self.folder_path)))
                .child(
                    Button::new("add-folder-path")
                        .label("Add path")
                        .disabled(self.scanning)
                        .on_click(cx.listener(|this, _, window, cx| {
                            let path = this.folder_path.read(cx).value().trim().to_owned();
                            if path.is_empty() {
                                this.status = "Enter a music folder path.".into();
                                cx.notify();
                                return;
                            }
                            this.library.cancel.store(false, Ordering::Relaxed);
                            this.scanning = true;
                            this.send_library(LibraryCommand::AddFolderAndScan(PathBuf::from(
                                path,
                            )));
                            this.folder_path
                                .update(cx, |input, cx| input.set_value("", window, cx));
                            cx.notify();
                        })),
                ),
        );
        for (index, root) in self.snapshot.roots.iter().enumerate() {
            let root = root.clone();
            let edit_root = root.clone();
            body = body.child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(Icon::new(IconName::Folder))
                    .child(div().flex_1().min_w_0().child(root.display().to_string()))
                    .child(
                        Button::new(("folder-edit", index))
                            .ghost()
                            .label("Relocate")
                            .disabled(self.scanning)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.folder_path.update(cx, |input, cx| {
                                    input.set_value(edit_root.display().to_string(), window, cx)
                                });
                                this.overlay = Some(Overlay::RelocateFolder(edit_root.clone()));
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new(("folder-remove", index))
                            .ghost()
                            .label("Remove")
                            .disabled(self.scanning)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.overlay = Some(Overlay::RemoveFolder(root.clone()));
                                cx.notify();
                            })),
                    ),
            );
        }
        body = body.child(self.render_filters(cx));
        body = body.child(
            div()
                .mt_6()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!("Library data: {}", self.paths.data.display())),
        );
        if !self.warnings.is_empty() {
            body = body.child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Library warnings"),
            );
            for warning in &self.warnings {
                body = body.child(div().text_sm().child(warning.clone()));
            }
        }
        body.overflow_y_scrollbar().into_any_element()
    }
}

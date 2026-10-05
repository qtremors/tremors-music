// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Confirmation dialogs, playlist selection, and track details.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_overlay(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let overlay = self.overlay.clone()?;
        let mut panel = div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x00000099))
            .child(div().absolute().inset_0().on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.overlay = None;
                    cx.notify();
                }),
            ));
        let mut content = div()
            .id("overlay-panel")
            .w(px(440.))
            .max_h(px(500.))
            .overflow_y_scroll()
            .p_6()
            .rounded_xl()
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .flex()
            .flex_col()
            .gap_3();
        match overlay {
            Overlay::RelocateFolder(old) => {
                content=content.child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child("Relocate music folder"))
                    .child("Choose the new location for this folder. Tracks found at matching relative paths keep their playlists and favorites.")
                    .child(Input::new(&self.folder_path))
                    .child(Button::new("confirm-relocate").primary().label("Save and scan").on_click(cx.listener(move |this,_,_,cx| {
                        let path=this.folder_path.read(cx).value().trim().to_owned();
                        if path.is_empty() {return;}
                        this.library.cancel.store(false,Ordering::Relaxed);this.scanning=true;
                        this.send_library(LibraryCommand::RelocateAndScan(old.clone(),PathBuf::from(path)));this.overlay=None;cx.notify();
                    })));
            }

            Overlay::Reset(all) => {
                content=content.child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child("Reset your library?"))
                    .child("Playlists, favorites and play counts will be removed. Your music files remain untouched.")
                    .child(Button::new("confirm-reset").danger().label("Reset library").on_click(cx.listener(move |this,_,_,cx| {
                        this.queue.clear();this.stop();this.send_library(LibraryCommand::Reset(all));this.overlay=None;cx.notify();
                    })));
            }
            Overlay::Details(song) => {
                content = content
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(song.title.clone()),
                    )
                    .child(format!("{} · {}", song.artist, song.album))
                    .child(song.path.display().to_string())
                    .child(format!(
                        "{} · {:.1} MiB · {}",
                        song.details.format,
                        song.details.file_size as f64 / 1048576.,
                        duration_label(song.duration)
                    ))
                    .child(format!(
                        "{} Hz · {} channels · {} kbps · {} bit",
                        song.details.sample_rate.unwrap_or(0),
                        song.details.channels.unwrap_or(0),
                        song.details.bitrate.unwrap_or(0),
                        song.details.bit_depth.unwrap_or(0)
                    ))
                    .child(format!(
                        "Year: {} · Track: {} · Disc: {} · Genre: {}",
                        song.year.unwrap_or(0),
                        song.track.unwrap_or(0),
                        song.disc.unwrap_or(0),
                        song.genre
                    ))
                    .child(format!("Played {} times", song.play_count));
                for (key, value) in song.details.tags {
                    content = content.child(format!("{key}: {value}"));
                }
            }

            Overlay::AddToPlaylist(song) => {
                content = content.child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Add to playlist"),
                );
                if self.snapshot.playlists.is_empty() {
                    content = content.child("Create a playlist in the sidebar first.");
                }
                for pl in &self.snapshot.playlists {
                    let id = pl.id;
                    content = content.child(
                        Button::new(("pick-playlist", id as usize))
                            .label(pl.name.clone())
                            .w_full()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.send_library(LibraryCommand::AddToPlaylist(id, song));
                                this.overlay = None;
                                cx.notify();
                            })),
                    );
                }
            }
            Overlay::DeletePlaylist(id) => {
                content = content
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Delete this playlist?"),
                    )
                    .child("This removes the playlist. Your music files remain untouched.")
                    .child(
                        Button::new("confirm-delete")
                            .danger()
                            .label("Delete playlist")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.send_library(LibraryCommand::DeletePlaylist(id));
                                this.overlay = None;
                                cx.notify();
                            })),
                    );
            }
            Overlay::RemoveFolder(root) => {
                content=content.child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child("Remove this music folder?"))
                    .child(root.display().to_string()).child("Tracks unique to this folder will leave your library and playlists. Files on disk remain untouched.")
                    .child(Button::new("confirm-remove-folder").danger().label("Remove folder")
                        .on_click(cx.listener(move |this,_,_,cx| {this.send_library(LibraryCommand::RemoveFolder(root.clone()));this.overlay=None;cx.notify();})));
            }
        }
        content = content.child(
            Button::new("close-overlay")
                .ghost()
                .label("Cancel")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.overlay = None;
                    cx.notify();
                })),
        );
        panel = panel.child(content);
        Some(panel.into_any_element())
    }
}

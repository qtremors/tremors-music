// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Native pickers only select paths; worker commands perform all file I/O.
use super::*;
#[derive(Clone)]
pub(super) enum FileTask {
    ImportPlaylist,
    ExportPlaylist(i64, bool),
    Artwork(String),
    MetadataPicture(i64),
    ExcludeFolder,
}
impl MusicApp {
    pub(in crate::ui) fn pick_file(&mut self, task: FileTask) {
        if self.picking_file {
            return;
        }
        self.picking_file = true;
        let sender = self.file_tx.clone();
        std::thread::spawn(move || {
            let dialog = rfd::FileDialog::new();
            let path = match &task {
                FileTask::ImportPlaylist => dialog
                    .add_filter("Local playlists", &["m3u", "m3u8"])
                    .pick_file(),
                FileTask::ExportPlaylist(_, _) => dialog
                    .add_filter("UTF-8 playlist", &["m3u8"])
                    .set_file_name("playlist.m3u8")
                    .save_file(),
                FileTask::Artwork(_) | FileTask::MetadataPicture(_) => dialog
                    .add_filter("Artwork", &["jpg", "jpeg", "png", "webp"])
                    .pick_file(),
                FileTask::ExcludeFolder => dialog.pick_folder(),
            };
            let _ = sender.send((task, path));
        });
    }
    pub(in crate::ui) fn poll_files(&mut self) -> bool {
        let mut changed = false;
        while let Ok((task, path)) = self.file_rx.try_recv() {
            changed = true;
            self.picking_file = false;
            let Some(path) = path else {
                continue;
            };
            match task {
                FileTask::ImportPlaylist => {
                    self.status = "Importing playlist…".into();
                    self.send_library(LibraryCommand::ImportPlaylist(path));
                }
                FileTask::ExportPlaylist(id, relative) => {
                    let path = if path.extension().is_none() {
                        path.with_extension("m3u8")
                    } else {
                        path
                    };
                    self.send_library(LibraryCommand::ExportPlaylist(id, path, relative))
                }
                FileTask::Artwork(key) => {
                    self.send_library(LibraryCommand::SetArtwork(key, Some(path)))
                }
                FileTask::MetadataPicture(id) if self.page == Page::Metadata(id) => {
                    self.send_library(LibraryCommand::LoadPicture(id, path))
                }
                FileTask::MetadataPicture(_) => {}
                FileTask::ExcludeFolder => {
                    if !self.preferences.excluded_folders.contains(&path) {
                        self.preferences.excluded_folders.push(path);
                        self.persist_modes();
                    }
                }
            }
        }
        changed
    }
}

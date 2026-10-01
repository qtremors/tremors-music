// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Navigation, folder selection, and playlist naming actions.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn start_scan(&mut self) {
        if self.scanning {
            return;
        }
        self.library.cancel.store(false, Ordering::Relaxed);
        self.scanning = true;
        self.send_library(LibraryCommand::Scan);
    }

    pub(in crate::ui) fn pick_folder(&mut self) {
        if self.picking_folder || self.scanning {
            return;
        }
        self.picking_folder = true;
        let tx = self.folder_tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(
                rfd::FileDialog::new()
                    .set_title("Choose a music folder")
                    .pick_folder(),
            );
        });
    }

    pub(in crate::ui) fn navigate(
        &mut self,
        page: Page,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        if self.page != page {
            self.history.push(self.page.clone());
        }
        self.page = page;
        self.query.clear();
        self.overlay = None;
        self.search
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.playlist_name
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.rebuild();
        cx.notify();
    }

    pub(in crate::ui) fn save_playlist(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.playlist_name.read(cx).value().to_string();
        if let Page::Playlist(id) = self.page {
            self.send_library(LibraryCommand::RenamePlaylist(id, name));
        } else {
            self.send_library(LibraryCommand::CreatePlaylist(name));
        }
        self.playlist_name
            .update(cx, |input, cx| input.set_value("", window, cx));
        cx.notify();
    }

    pub(in crate::ui) fn title(&self) -> String {
        match &self.page {
            Page::Library => "Songs".into(),
            Page::Albums => "Albums".into(),
            Page::Artists => "Artists".into(),
            Page::Genres => "Genres".into(),
            Page::Genre(genre) => genre.clone(),
            Page::Playlists => "Playlists".into(),
            Page::Search => "Search your library".into(),
            Page::Settings => "Settings".into(),
            Page::Lyrics => "Lyrics".into(),
            Page::Favorites => "Favorites".into(),
            Page::Recent => "Recently added".into(),
            Page::MostPlayed => "Most played".into(),
            Page::Album(album, _) => album.clone(),
            Page::Artist(artist) => artist.clone(),
            Page::Playlist(id) => self
                .snapshot
                .playlists
                .iter()
                .find(|p| p.id == *id)
                .map(|p| p.name.clone())
                .unwrap_or_default(),
            Page::Queue => "Up next".into(),
            Page::Folders => "Music folders".into(),
        }
    }
}

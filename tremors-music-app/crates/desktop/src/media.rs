// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use gpui_kit::Window;
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use std::{
    sync::mpsc::{self, Receiver},
    time::Duration,
};
use tremors_core::model::Song;

pub fn open(window: &Window) -> (Option<MediaControls>, Receiver<MediaControlEvent>) {
    let (tx, rx) = mpsc::channel();
    #[cfg(windows)]
    let hwnd = {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        HasWindowHandle::window_handle(window)
            .ok()
            .and_then(|handle| match handle.as_raw() {
                RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as *mut std::ffi::c_void),
                _ => None,
            })
    };
    #[cfg(not(windows))]
    let hwnd = {
        let _ = window;
        None
    };
    let controls = (|| -> Result<_, souvlaki::Error> {
        let mut controls = MediaControls::new(PlatformConfig {
            display_name: "Tremors Music",
            dbus_name: "tremorsmusic",
            hwnd,
        })?;
        controls.attach(move |event| {
            let _ = tx.send(event);
        })?;
        Ok(controls)
    })()
    .ok();
    (controls, rx)
}

pub fn metadata(controls: &mut MediaControls, song: Option<&Song>) {
    let cover = song
        .and_then(|s| s.cover.as_ref())
        .and_then(|path| url::Url::from_file_path(path).ok());
    let _ = controls.set_metadata(MediaMetadata {
        title: song.map(|s| s.title.as_str()),
        artist: song.map(|s| s.artist.as_str()),
        album: song.map(|s| s.album.as_str()),
        cover_url: cover.as_ref().map(|u| u.as_str()),
        duration: song.map(|s| Duration::from_secs_f64(s.duration.max(0.))),
    });
}

pub fn playback(controls: &mut MediaControls, has_track: bool, playing: bool, position: f64) {
    let progress = Some(MediaPosition(Duration::from_secs_f64(position.max(0.))));
    let _ = controls.set_playback(if !has_track {
        MediaPlayback::Stopped
    } else if playing {
        MediaPlayback::Playing { progress }
    } else {
        MediaPlayback::Paused { progress }
    });
}

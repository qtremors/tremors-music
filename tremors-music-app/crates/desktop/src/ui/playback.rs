// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Playback transitions and persistence of queue state.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn cache_lyrics(&mut self) {
        self.lyric_lines = Arc::new(
            self.queue
                .current()
                .map(|s| parse_lrc(&s.details.lyrics))
                .unwrap_or_default(),
        );
        self.lyric_index = None;
    }

    pub(in crate::ui) fn persist_queue(&mut self) {
        self.send_library(LibraryCommand::Session(self.queue.state(self.position)));
        self.saved_at = Instant::now();
    }

    pub(in crate::ui) fn load_current(&mut self) {
        self.load_current_at(0.);
    }

    pub(in crate::ui) fn load_current_at(&mut self, position: f64) {
        if let Some(song) = self.queue.current().cloned() {
            self.generation += 1;
            self.position = position.max(0.).min(song.duration.max(0.));
            self.duration = song.duration;
            self.is_playing = false;
            self.loading = true;
            self.loaded = false;
            self.ended = false;
            self.seeking = false;
            self.send_audio(AudioCommand::Load {
                song_id: song.id,
                path: song.path,
                generation: self.generation,
                start: Duration::from_secs_f64(self.position),
                gain: tremors_core::playback::track_gain(
                    &song.details,
                    self.preferences.replay_gain,
                ),
            });
            self.cache_lyrics();
            self.persist_queue();
        }
    }

    pub(in crate::ui) fn toggle(&mut self) {
        if self.loading {
            return;
        }
        if self.queue.current().is_none() {
            let songs = self.visible_songs();
            if !songs.is_empty() {
                self.queue.replace(songs, 0);
                self.load_current();
            }
        } else if self.ended || !self.loaded {
            self.load_current_at(if self.ended { 0. } else { self.position });
        } else if !self.loading {
            self.is_playing = !self.is_playing;
            self.send_audio(if self.is_playing {
                AudioCommand::Resume
            } else {
                AudioCommand::Pause
            });
            self.persist_queue();
        }
    }

    pub(in crate::ui) fn next(&mut self) {
        if self.queue.next(false).is_some() {
            self.load_current();
        }
        self.rebuild();
    }

    pub(in crate::ui) fn previous(&mut self) {
        if self.position > 3. && !self.ended {
            self.send_audio(AudioCommand::Seek(Duration::ZERO));
            self.position = 0.;
        } else if self.queue.previous().is_some() {
            self.load_current();
        }
        self.rebuild();
    }

    pub(in crate::ui) fn persist_modes(&mut self) {
        self.preferences.shuffle = self.queue.shuffle;
        self.preferences.repeat = self.queue.repeat;
        self.send_library(LibraryCommand::Preferences(self.preferences.clone()));
        self.persist_queue();
    }

    pub(in crate::ui) fn stop(&mut self) {
        self.generation += 1;
        self.send_audio(AudioCommand::Stop);
        self.is_playing = false;
        self.loading = false;
        self.loaded = false;
        self.position = 0.;
        self.duration = self.queue.current().map_or(0., |s| s.duration);
        self.ended = false;
        self.cache_lyrics();
        self.persist_queue();
    }

    pub(in crate::ui) fn seek_to(&mut self, seconds: f64) {
        if self.queue.current().is_none() {
            return;
        }
        let target = seconds.clamp(0., self.duration.max(0.));
        if !self.loaded || self.ended {
            self.load_current_at(target);
        } else {
            self.position = target;
            self.send_audio(AudioCommand::Seek(Duration::from_secs_f64(target)));
            self.persist_queue();
        }
    }

    pub(in crate::ui) fn set_volume(
        &mut self,
        volume: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.preferences.volume = volume.clamp(0., 1.);
        self.muted = false;
        self.send_audio(AudioCommand::Volume(self.preferences.volume));
        #[cfg(target_os = "linux")]
        if let Some(media) = &mut self.media {
            let _ = media.set_volume(self.preferences.volume as f64);
        }
        self.volume.update(cx, |slider, cx| {
            slider.set_value(self.preferences.volume * 100., window, cx)
        });
        self.persist_modes();
        cx.notify();
    }

    pub(in crate::ui) fn mute(&mut self, cx: &mut Context<Self>) {
        self.muted = !self.muted;
        #[cfg(target_os = "linux")]
        if let Some(media) = &mut self.media {
            let _ = media.set_volume(if self.muted {
                0.
            } else {
                self.preferences.volume as f64
            });
        }
        self.send_audio(AudioCommand::Volume(if self.muted {
            0.
        } else {
            self.preferences.volume
        }));
        cx.notify();
    }

    pub(in crate::ui) fn fullscreen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.immersive = !self.immersive;
        self.lyric_index = None;
        window.toggle_fullscreen();
        cx.notify();
    }
}

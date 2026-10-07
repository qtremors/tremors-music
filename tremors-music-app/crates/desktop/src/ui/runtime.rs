// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Worker composition, event handling, initialization, and shutdown.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn new(paths: AppPaths, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (media, media_rx) = crate::media::open(window);
        let library = service::spawn(paths.clone());
        let audio = audio::spawn();
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search songs, artists, albums…"));
        let folder_path = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Folder path, e.g. C:\\Music or /home/you/Music")
        });
        let playlist_name = cx.new(|cx| InputState::new(window, cx).placeholder("Playlist name"));
        let seek = cx.new(|_| SliderState::new().min(0.).max(100.).step(0.1));
        let volume = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(100.)
                .step(1.)
                .default_value(70.)
        });
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let (folder_tx, folder_rx) = mpsc::channel();
        let subscriptions = vec![
            cx.subscribe_in(&search, window, |this, state, event, _, cx| {
                if matches!(event, InputEvent::Change) {
                    this.query = state.read(cx).value().to_string();
                    this.rebuild();
                    cx.notify();
                }
            }),
            cx.subscribe_in(&playlist_name, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.save_playlist(window, cx);
                }
            }),
            cx.subscribe_in(&seek, window, |this, _, event, _, cx| {
                match event {
                    SliderEvent::Change(SliderValue::Single(v)) => {
                        this.seeking = true;
                        this.position = this.duration * (*v as f64 / 100.);
                    }
                    SliderEvent::Release(SliderValue::Single(v)) => {
                        let position = this.duration * (*v as f64 / 100.);
                        if this.ended {
                            this.load_current();
                        }
                        if this.loaded || this.loading {
                            this.position = position;
                            this.send_audio(AudioCommand::Seek(Duration::from_secs_f64(
                                position.max(0.),
                            )));
                        }
                        this.persist_queue();
                        this.seeking = false;
                    }
                    _ => {}
                }
                cx.notify();
            }),
            cx.subscribe_in(&volume, window, |this, _, event, _, cx| {
                match event {
                    SliderEvent::Change(SliderValue::Single(v)) => {
                        this.muted = false;
                        this.preferences.volume = *v / 100.;
                        this.send_audio(AudioCommand::Volume(*v / 100.));
                        #[cfg(target_os = "linux")]
                        if let Some(media) = &mut this.media {
                            let _ = media.set_volume((*v / 100.) as f64);
                        }
                    }
                    SliderEvent::Release(SliderValue::Single(v)) => {
                        this.preferences.volume = *v / 100.;
                        this.muted = false;
                        this.send_library(LibraryCommand::Preferences(this.preferences.clone()));
                    }
                    _ => {}
                }
                cx.notify();
            }),
        ];
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                if this
                    .update_in(cx, |this, window, cx| this.poll(window, cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        Self {
            paths,
            library,
            audio,
            folder_rx,
            folder_tx,
            picking_folder: false,
            snapshot: Arc::new(Snapshot::default()),
            initialized: false,
            preferences: Preferences::default(),
            loaded: false,
            page: Page::Library,
            rows: Vec::new(),
            query: String::new(),
            search,
            playlist_name,
            folder_path,
            seek,
            volume,
            focus,
            _subscriptions: subscriptions,
            queue: Queue::default(),
            generation: 0,
            is_playing: false,
            loading: false,
            ended: false,
            position: 0.,
            duration: 0.,
            seeking: false,
            scanning: false,
            status: "Loading your library…".into(),
            warnings: Vec::new(),
            overlay: None,
            history: Vec::new(),
            replacing_folder: None,
            saved_at: Instant::now(),
            muted: false,
            immersive: false,
            lyrics_scroll: UniformListScrollHandle::new(),
            lyric_index: None,
            lyric_lines: Arc::new(Vec::new()),
            media,
            media_rx,
            media_song: None,
            media_status: (false, false, u64::MAX),
        }
    }

    pub(in crate::ui) fn send_library(&mut self, command: LibraryCommand) {
        if self.library.commands.send(command).is_err() {
            self.status = "Library worker stopped. Restart the app.".into();
        }
    }

    pub(in crate::ui) fn send_audio(&mut self, command: AudioCommand) {
        if self.audio.commands.send(command).is_err() {
            self.status = "Audio worker stopped. Restart the app.".into();
            self.is_playing = false;
        }
    }

    pub(in crate::ui) fn poll(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut changed = false;
        while let Ok(event) = self.library.events.try_recv() {
            changed = true;
            match event {
                LibraryEvent::Snapshot(snapshot) => {
                    if !self.initialized {
                        self.preferences = snapshot.preferences.clone();
                        self.queue.repeat = snapshot.preferences.repeat;
                        self.queue.shuffle = snapshot.preferences.shuffle;
                        self.position = self.queue.restore(&snapshot.session, &snapshot.songs);
                        self.duration = self.queue.current().map_or(0., |s| s.duration);
                        self.ended = self.duration > 0. && self.position >= self.duration;
                        self.cache_lyrics();
                        self.apply_theme(window, cx);
                        let volume = snapshot.preferences.volume.clamp(0., 1.);
                        self.send_audio(AudioCommand::Volume(volume));
                        self.volume
                            .update(cx, |slider, cx| slider.set_value(volume * 100., window, cx));
                        self.status = if snapshot.roots.is_empty() {
                            "Add a music folder to get started.".into()
                        } else {
                            "Your library is ready.".into()
                        };
                        self.initialized = true;
                    }
                    let removed = self.queue.reconcile(&snapshot.songs);
                    if removed {
                        self.stop();
                    }
                    self.cache_lyrics();
                    self.snapshot = snapshot;
                    if let Page::Playlist(id) = self.page
                        && !self.snapshot.playlists.iter().any(|pl| pl.id == id)
                    {
                        self.page = Page::Library;
                    }
                    self.rebuild();
                }
                LibraryEvent::ScanStarted => {
                    self.scanning = true;
                    self.warnings.clear();
                    self.status = "Scanning music folders…".into();
                }
                LibraryEvent::ScanProgress(p) => {
                    self.status = format!(
                        "Scanning: {} tracks read · {} skipped · {}",
                        p.imported, p.skipped, p.current
                    )
                }
                LibraryEvent::ScanFinished { message, warnings } => {
                    self.scanning = false;
                    self.status = message;
                    self.warnings = warnings;
                }
                LibraryEvent::Warning(message) => {
                    self.warnings.push(message);
                }
                LibraryEvent::Error(message) => {
                    self.status = message;
                    self.scanning = false;
                }
            }
        }
        while let Ok(event) = self.audio.events.try_recv() {
            match event {
                AudioEvent::Loaded {
                    song_id,
                    generation,
                    duration,
                } if generation == self.generation => {
                    self.loading = false;
                    self.loaded = true;
                    self.is_playing = true;
                    self.ended = false;
                    if let Some(duration) = duration {
                        self.duration = duration.as_secs_f64();
                    }
                    self.send_library(LibraryCommand::Played(song_id));
                    changed = true;
                }
                AudioEvent::Position {
                    generation,
                    position,
                    paused,
                } if generation == self.generation && !self.ended => {
                    let previous_position = self.position;
                    let previous_playing = self.is_playing;
                    if !self.seeking {
                        self.position = position.as_secs_f64();
                    }
                    self.is_playing = !paused;
                    changed |=
                        self.position != previous_position || self.is_playing != previous_playing;
                }
                AudioEvent::Finished { generation } if generation == self.generation => {
                    self.is_playing = false;
                    self.ended = true;
                    self.position = self.duration;
                    if self.queue.next(true).is_some() {
                        self.load_current();
                    }
                    self.rebuild();
                    changed = true;
                }
                AudioEvent::Error {
                    generation,
                    message,
                    fatal,
                } if generation == self.generation => {
                    self.status = message;
                    if fatal {
                        self.loading = false;
                        self.loaded = false;
                        self.is_playing = false;
                    }
                    changed = true;
                }
                _ => {}
            }
        }
        while let Ok(folder) = self.folder_rx.try_recv() {
            self.picking_folder = false;
            changed = true;
            if let Some(folder) = folder {
                if let Some(old) = self.replacing_folder.take() {
                    self.send_library(LibraryCommand::RelocateAndScan(old, folder));
                } else {
                    self.send_library(LibraryCommand::AddFolderAndScan(folder));
                }
                self.library.cancel.store(false, Ordering::Relaxed);
                self.scanning = true;
            } else {
                self.replacing_folder = None;
            }
        }
        while let Ok(event) = self.media_rx.try_recv() {
            use souvlaki::{MediaControlEvent as Event, SeekDirection};
            match event {
                Event::Toggle => self.toggle(),
                Event::Play if !self.is_playing => self.toggle(),
                Event::Pause if self.is_playing => self.toggle(),
                Event::Next => self.next(),
                Event::Previous => self.previous(),
                Event::Stop => self.stop(),
                Event::Seek(direction) => self.seek_to(
                    self.position
                        + if direction == SeekDirection::Forward {
                            10.
                        } else {
                            -10.
                        },
                ),
                Event::SeekBy(direction, duration) => self.seek_to(
                    self.position
                        + duration.as_secs_f64()
                            * if direction == SeekDirection::Forward {
                                1.
                            } else {
                                -1.
                            },
                ),
                Event::SetPosition(position) => self.seek_to(position.0.as_secs_f64()),
                Event::SetVolume(volume) if volume.is_finite() => {
                    self.set_volume(volume as f32, window, cx)
                }
                Event::Raise => {
                    window.activate_window();
                    cx.activate(true);
                }
                Event::Quit => cx.quit(),
                _ => {}
            }
            self.rebuild();
            changed = true;
        }
        let song = self.queue.current();
        let id = song.map(|s| s.id);
        if let Some(media) = &mut self.media {
            if id != self.media_song {
                crate::media::metadata(media, song);
                self.media_song = id;
            }
            let status = (
                song.is_some(),
                self.is_playing,
                self.position.max(0.) as u64,
            );
            if status != self.media_status {
                crate::media::playback(media, song.is_some(), self.is_playing, self.position);
                self.media_status = status;
            }
        }
        if self.initialized && self.is_playing && self.saved_at.elapsed() >= Duration::from_secs(5)
        {
            self.persist_queue();
        }
        if self.page == Page::Lyrics || self.immersive {
            let active = active_line(&self.lyric_lines, self.position);
            if active != self.lyric_index {
                self.lyric_index = active;
                if let Some(index) = active {
                    self.lyrics_scroll
                        .scroll_to_item(index, ScrollStrategy::Center);
                }
                changed = true;
            }
        }
        if changed {
            if !self.seeking {
                let percent = if self.duration > 0. {
                    ((self.position / self.duration) * 100.).clamp(0., 100.) as f32
                } else {
                    0.
                };
                self.seek
                    .update(cx, |slider, cx| slider.set_value(percent, window, cx));
            }
            cx.notify();
        }
    }
}

impl Drop for MusicApp {
    fn drop(&mut self) {
        if !self.initialized {
            return;
        }
        self.library.cancel.store(true, Ordering::Relaxed);
        // Save synchronously at shutdown: a queued write could be interrupted when the process exits.
        let result = (|| -> anyhow::Result<()> {
            let mut library = tremors_core::library::Library::open(&self.paths.database)?;
            library.save_session(&self.queue.state(self.position))?;
            library.save_preferences(&self.preferences)
        })();
        if let Err(error) = result {
            eprintln!("Could not save playback state at shutdown: {error:#}");
        }
    }
}

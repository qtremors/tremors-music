// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Worker composition, event handling, initialization, and shutdown.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn new(paths: AppPaths, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            view.update(cx, |this, cx| {
                if this.metadata_busy() {
                    this.status = "Finish saving your edits before closing.".into();
                    cx.notify();
                    false
                } else if this.draft_dirty() {
                    this.open_overlay(Overlay::CloseUnsaved, window, cx);
                    false
                } else {
                    true
                }
            })
            .unwrap_or(true)
        });
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
        let overlay_focus = cx.focus_handle();
        let lyrics_scroll = ListState::new(0, ListAlignment::Top, px(200.));
        let view = cx.entity().downgrade();
        lyrics_scroll.set_scroll_handler(move |_, _, cx| {
            let _ = view.update(cx, |this, cx| {
                this.lyric_follow = false;
                cx.notify();
            });
        });
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let (folder_tx, folder_rx) = mpsc::channel();
        let (file_tx, file_rx) = mpsc::channel();
        let minimum_duration =
            cx.new(|cx| InputState::new(window, cx).placeholder("Minimum seconds"));
        let subscriptions = vec![
            cx.subscribe_in(&search, window, |this, state, event, _, cx| {
                if matches!(event, InputEvent::Change)
                    && !matches!(this.page, Page::Metadata(_) | Page::LyricsEditor(_))
                {
                    let query = state.read(cx).value().to_string();
                    if query == this.query {
                        return;
                    }
                    if !query.trim().is_empty() && this.page != Page::Search {
                        this.remember_scroll();
                        this.history.push(this.page.clone());
                        this.page = Page::Search;
                        this.restore_scroll();
                        this.page_revision += 1;
                    }
                    this.selected_songs.clear();
                    this.query = query;
                    this.table_scroll = UniformListScrollHandle::new();
                    this.rebuild();
                    cx.notify();
                }
            }),
            cx.subscribe_in(&playlist_name, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.save_playlist(window, cx);
                } else if matches!(event, InputEvent::Change) {
                    cx.notify();
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
            backup_review: None,
            selection_mode: false,
            selected_songs: Default::default(),
            metadata: None,
            lyric_editor: None,
            file_tx,
            file_rx,
            picking_file: false,
            minimum_duration,
            sleep_timer: None,
            timer_finish_track: false,
            timer_fade: true,
            timer_label: String::new(),
            timer_gain: 1.,
            page_revision: 0,
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
            page: Page::Home,
            rows: Vec::new(),
            home_sections: Vec::new(),
            artist_collections: Vec::new(),
            show_artist_albums: true,
            gallery_columns: 4,
            page_scroll: ScrollHandle::new(),
            saved_scroll: Vec::new(),
            home_scroll: Default::default(),
            overlay_focus,
            overlay_return_focus: None,
            overlay_scroll: ScrollHandle::new(),
            table_scroll: UniformListScrollHandle::new(),
            gallery_scroll: UniformListScrollHandle::new(),
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
            prepared_next: None,
            audio_edit_pending: false,
            next_token: 0,
            sound: audio_settings::Controls::new(window, cx),
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
            lyrics_scroll,
            lyric_plain_lines: Arc::new(Vec::new()),
            lyrics_source: None,
            lyric_follow: true,
            lyric_index: None,
            lyric_lines: Arc::new(Vec::new()),
            media,
            media_rx,
            media_revision: None,
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
                LibraryEvent::BackupLoaded(document) => self.review_backup(*document, window, cx),
                LibraryEvent::BackupRestored {
                    preferences,
                    warnings,
                } => {
                    self.status = "Library curation restored.".into();
                    self.warnings = warnings;
                    if let Some(preferences) = preferences {
                        self.preferences = preferences;
                        self.initialize_sound(window, cx);
                        self.queue.set_shuffle(self.preferences.shuffle);
                        self.queue.repeat = self.preferences.repeat;
                        self.apply_theme(window, cx);
                        self.set_volume(self.preferences.volume, window, cx);
                    }
                }
                LibraryEvent::Snapshot(snapshot) => {
                    if !self.initialized {
                        self.preferences = snapshot.preferences.clone();
                        self.initialize_sound(window, cx);
                        self.minimum_duration.update(cx, |input, cx| {
                            input.set_value(
                                self.preferences.minimum_duration.to_string(),
                                window,
                                cx,
                            )
                        });
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
                    self.home_sections = home_sections(&snapshot);
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
                LibraryEvent::LyricsLoaded(document) => self.receive_lyrics(document, window, cx),
                LibraryEvent::LyricsError(id, message) => {
                    self.status = message.clone();
                    if let Some(editor) = &mut self.lyric_editor
                        && editor.document.song_id == id
                    {
                        editor.saving = false;
                        editor.message = message;
                    }
                }
                LibraryEvent::MetadataLoaded(document) => {
                    self.receive_metadata(*document, false, window, cx)
                }
                LibraryEvent::MetadataSaved(document) => {
                    self.receive_metadata(*document, true, window, cx);
                    self.media_revision = None;
                }
                LibraryEvent::MetadataError(id, message) => {
                    self.status = message.clone();
                    if self.page == Page::Metadata(id)
                        && let Some(editor) = &mut self.metadata
                    {
                        editor.saving = false;
                        editor.message = message;
                    }
                }
                LibraryEvent::PictureLoaded(id, picture) => {
                    self.append_picture(id, picture, window, cx)
                }
                LibraryEvent::OperationFinished { message, warnings } => {
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
                AudioEvent::EditReady {
                    generation,
                    released,
                } => {
                    self.audio_edit_pending = false;
                    if released && generation == self.generation {
                        self.prepared_next = None;
                        self.loaded = false;
                        self.loading = false;
                        self.is_playing = false;
                        changed = true;
                    }
                }
                AudioEvent::Advanced {
                    generation,
                    track,
                    duration,
                } if generation == self.generation => {
                    if self.prepared_next.as_ref().is_some_and(|next| {
                        next.token == track.token && next.song_id == track.song_id
                    }) && self.queue.next(true).is_some()
                    {
                        self.prepared_next = None;
                        self.position = 0.;
                        self.duration = duration
                            .map(|d| d.as_secs_f64())
                            .unwrap_or_else(|| self.queue.current().map_or(0., |s| s.duration));
                        self.loading = false;
                        self.loaded = true;
                        self.ended = false;
                        self.cache_lyrics();
                        self.persist_queue();
                        self.send_library(LibraryCommand::Played(track.song_id));
                        self.rebuild();
                        changed = true;
                    } else {
                        self.load_current_at(self.position);
                    }
                }
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
                    if self
                        .sleep_timer
                        .as_ref()
                        .is_some_and(|t| t.stop_at_track_end(Instant::now()))
                    {
                        self.expire_timer();
                    } else if self.queue.next(true).is_some() {
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
        self.sync_preload();
        changed |= self.poll_files();
        changed |= self.poll_timer();
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
        let revision = song.map(crate::media::revision);
        if let Some(media) = &mut self.media {
            if revision != self.media_revision {
                crate::media::metadata(media, song);
                self.media_revision = revision;
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
            let active = active_line(&self.lyric_lines, self.lyric_position());
            if active != self.lyric_index {
                for index in self.lyric_index.into_iter().chain(active) {
                    self.lyrics_scroll.remeasure_items(index..index + 1);
                }
                self.lyric_index = active;
                if self.lyric_follow
                    && let Some(index) = active
                {
                    self.lyrics_scroll.scroll_to_reveal_item(index);
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

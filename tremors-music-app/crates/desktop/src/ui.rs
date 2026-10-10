// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, Selectable, Sizable, Theme, ThemeMode,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    menu::{ContextMenuExt, DropdownMenu, PopupMenu, PopupMenuItem},
    slider::{Slider, SliderEvent, SliderState, SliderValue},
    switch::Switch,
};
use gpui_kit::{prelude::*, *};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::{Arc, atomic::Ordering, mpsc},
    time::{Duration, Instant},
};
use tremors_core::{
    browsing::{search_rank, sort_songs},
    lyrics::{active_line, parse_lrc},
    model::{Snapshot, Song, duration_label},
    playback::{self, AudioCommand, AudioEvent, AudioHandle},
    queue::Queue,
    service::{self, LibraryCommand, LibraryEvent, LibraryHandle},
    settings::{AppPaths, Preferences, RepeatMode, SongSort},
};

actions!(
    tremors,
    [
        TogglePlayback,
        NextTrack,
        PreviousTrack,
        FocusSearch,
        VolumeUp,
        VolumeDown,
        Mute,
        SeekForward,
        SeekBackward,
        ShowQueue,
        ShowLyrics,
        Fullscreen,
        Back,
        Escape
    ]
);

#[derive(Clone, PartialEq, Eq)]
enum Page {
    Library,
    Albums,
    Artists,
    Genres,
    Genre(String),
    Playlists,
    Search,
    Settings,
    Lyrics,
    Favorites,
    Recent,
    MostPlayed,
    Playlist(i64),
    Album(String, String),
    Artist(String),
    Queue,
    Folders,
}

#[derive(Clone)]
enum Row {
    Song {
        song: Arc<Song>,
        entry: Option<i64>,
        queue_index: Option<usize>,
    },
    Group {
        title: String,
        subtitle: String,
        cover: Option<PathBuf>,
        page: Page,
    },
}

#[derive(Clone)]
enum Overlay {
    AddToPlaylist(i64),
    DeletePlaylist(i64),
    RemoveFolder(PathBuf),
    Reset(bool),
    Details(Box<Song>),
    RelocateFolder(PathBuf),
}

#[derive(Clone)]
struct RowDrag {
    queue_index: Option<usize>,
    playlist: Option<(i64, i64)>,
    title: String,
}
impl Render for RowDrag {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_3()
            .rounded_md()
            .bg(cx.theme().muted)
            .text_color(cx.theme().foreground)
            .child(self.title.clone())
    }
}

struct MusicApp {
    paths: AppPaths,
    library: LibraryHandle,
    audio: AudioHandle,
    folder_rx: mpsc::Receiver<Option<PathBuf>>,
    folder_tx: mpsc::Sender<Option<PathBuf>>,
    picking_folder: bool,
    snapshot: Arc<Snapshot>,
    initialized: bool,
    preferences: Preferences,
    loaded: bool,
    page: Page,
    rows: Vec<Row>,
    query: String,
    search: Entity<InputState>,
    playlist_name: Entity<InputState>,
    folder_path: Entity<InputState>,
    seek: Entity<SliderState>,
    volume: Entity<SliderState>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
    queue: Queue,
    generation: u64,
    is_playing: bool,
    loading: bool,
    ended: bool,
    position: f64,
    duration: f64,
    seeking: bool,
    scanning: bool,
    status: String,
    warnings: Vec<String>,
    overlay: Option<Overlay>,
    history: Vec<Page>,
    replacing_folder: Option<PathBuf>,
    saved_at: Instant,
    muted: bool,
    immersive: bool,
    lyrics_scroll: UniformListScrollHandle,
    lyric_index: Option<usize>,
    lyric_lines: Arc<Vec<tremors_core::lyrics::LyricLine>>,
    media: Option<souvlaki::MediaControls>,
    media_rx: mpsc::Receiver<souvlaki::MediaControlEvent>,
    media_song: Option<i64>,
    media_status: (bool, bool, u64),
}

impl MusicApp {
    fn new(paths: AppPaths, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (media, media_rx) = crate::media::open(window);
        let library = service::spawn(paths.clone());
        let audio = playback::spawn();
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

    fn send_library(&mut self, command: LibraryCommand) {
        if self.library.commands.send(command).is_err() {
            self.status = "Library worker stopped. Restart the app.".into();
        }
    }

    fn send_audio(&mut self, command: AudioCommand) {
        if self.audio.commands.send(command).is_err() {
            self.status = "Audio worker stopped. Restart the app.".into();
            self.is_playing = false;
        }
    }

    fn poll(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    fn cache_lyrics(&mut self) {
        self.lyric_lines = Arc::new(
            self.queue
                .current()
                .map(|s| parse_lrc(&s.details.lyrics))
                .unwrap_or_default(),
        );
        self.lyric_index = None;
    }

    fn persist_queue(&mut self) {
        self.send_library(LibraryCommand::Session(self.queue.state(self.position)));
        self.saved_at = Instant::now();
    }

    fn load_current(&mut self) {
        self.load_current_at(0.);
    }
    fn load_current_at(&mut self, position: f64) {
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

    fn toggle(&mut self) {
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

    fn next(&mut self) {
        if self.queue.next(false).is_some() {
            self.load_current();
        }
        self.rebuild();
    }

    fn previous(&mut self) {
        if self.position > 3. && !self.ended {
            self.send_audio(AudioCommand::Seek(Duration::ZERO));
            self.position = 0.;
        } else if self.queue.previous().is_some() {
            self.load_current();
        }
        self.rebuild();
    }

    fn persist_modes(&mut self) {
        self.preferences.shuffle = self.queue.shuffle;
        self.preferences.repeat = self.queue.repeat;
        self.send_library(LibraryCommand::Preferences(self.preferences.clone()));
        self.persist_queue();
    }

    fn start_scan(&mut self) {
        if self.scanning {
            return;
        }
        self.library.cancel.store(false, Ordering::Relaxed);
        self.scanning = true;
        self.send_library(LibraryCommand::Scan);
    }

    fn pick_folder(&mut self) {
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

    fn navigate(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
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

    fn save_playlist(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    fn visible_songs(&self) -> Vec<Song> {
        self.rows
            .iter()
            .filter_map(|row| match row {
                Row::Song { song, .. } => Some((**song).clone()),
                _ => None,
            })
            .collect()
    }

    fn rebuild(&mut self) {
        self.rows.clear();
        let songs = &self.snapshot.songs;
        let by_id: HashMap<_, _> = songs.iter().map(|s| (s.id, s)).collect();
        match &self.page {
            Page::Albums => {
                let mut groups: BTreeMap<(String, String), Vec<&Song>> = BTreeMap::new();
                for s in songs {
                    groups
                        .entry((s.album.clone(), s.album_artist.clone()))
                        .or_default()
                        .push(s);
                }
                for ((album, artist), songs) in groups {
                    if !format!("{album} {artist}")
                        .to_lowercase()
                        .contains(&self.query.to_lowercase())
                    {
                        continue;
                    }
                    self.rows.push(Row::Group {
                        title: album.clone(),
                        subtitle: format!("{artist} · {} tracks", songs.len()),
                        cover: songs.iter().find_map(|s| s.cover.clone()),
                        page: Page::Album(album, artist),
                    });
                }
            }
            Page::Artists => {
                let mut groups: BTreeMap<String, Vec<&Song>> = BTreeMap::new();
                for s in songs {
                    groups.entry(s.artist.clone()).or_default().push(s);
                }
                for (artist, songs) in groups {
                    if !artist.to_lowercase().contains(&self.query.to_lowercase()) {
                        continue;
                    }
                    self.rows.push(Row::Group {
                        title: artist.clone(),
                        subtitle: format!("{} tracks", songs.len()),
                        cover: songs.iter().find_map(|s| s.cover.clone()),
                        page: Page::Artist(artist),
                    });
                }
            }
            Page::Genres => {
                let mut groups: BTreeMap<String, Vec<&Song>> = BTreeMap::new();
                for s in songs {
                    groups
                        .entry(if s.genre.is_empty() {
                            "Unknown genre".into()
                        } else {
                            s.genre.clone()
                        })
                        .or_default()
                        .push(s);
                }
                for (genre, songs) in groups {
                    if !genre.to_lowercase().contains(&self.query.to_lowercase()) {
                        continue;
                    }
                    self.rows.push(Row::Group {
                        title: genre.clone(),
                        subtitle: format!("{} tracks", songs.len()),
                        cover: songs.iter().find_map(|s| s.cover.clone()),
                        page: Page::Genre(genre),
                    });
                }
            }
            Page::Playlists => {
                for pl in &self.snapshot.playlists {
                    if !pl.name.to_lowercase().contains(&self.query.to_lowercase()) {
                        continue;
                    }
                    self.rows.push(Row::Group {
                        title: pl.name.clone(),
                        subtitle: format!("{} tracks", pl.entries.len()),
                        cover: pl
                            .entries
                            .iter()
                            .filter_map(|e| by_id.get(&e.song_id))
                            .find_map(|s| s.cover.clone()),
                        page: Page::Playlist(pl.id),
                    });
                }
            }
            Page::Search => {
                if self.query.trim().is_empty() {
                    return;
                }
                let q = self.query.to_lowercase();
                let mut albums = BTreeMap::new();
                let mut artists = BTreeMap::new();
                for s in songs {
                    if format!("{} {}", s.album, s.album_artist)
                        .to_lowercase()
                        .contains(&q)
                    {
                        albums
                            .entry((s.album.clone(), s.album_artist.clone()))
                            .or_insert(s.cover.clone());
                    }
                    if s.artist.to_lowercase().contains(&q) {
                        artists.entry(s.artist.clone()).or_insert(s.cover.clone());
                    }
                }
                for (artist, cover) in artists {
                    self.rows.push(Row::Group {
                        title: artist.clone(),
                        subtitle: "Artist".into(),
                        cover,
                        page: Page::Artist(artist),
                    });
                }
                for ((album, artist), cover) in albums {
                    self.rows.push(Row::Group {
                        title: album.clone(),
                        subtitle: format!("Album · {artist}"),
                        cover,
                        page: Page::Album(album, artist),
                    });
                }
                let mut selected: Vec<_> = songs
                    .iter()
                    .filter(|s| s.matches(&self.query))
                    .cloned()
                    .collect();
                selected
                    .sort_by_cached_key(|s| (search_rank(s, &self.query), s.title.to_lowercase()));
                self.rows.extend(selected.into_iter().map(|song| Row::Song {
                    song: Arc::new(song),
                    entry: None,
                    queue_index: None,
                }));
            }
            Page::Playlist(id) => {
                if let Some(pl) = self.snapshot.playlists.iter().find(|pl| pl.id == *id) {
                    for entry in &pl.entries {
                        if let Some(song) =
                            by_id.get(&entry.song_id).filter(|s| s.matches(&self.query))
                        {
                            self.rows.push(Row::Song {
                                song: Arc::new((**song).clone()),
                                entry: Some(entry.id),
                                queue_index: None,
                            });
                        }
                    }
                }
            }
            Page::Queue => {
                for (index, song) in self.queue.ordered().into_iter().enumerate() {
                    if song.matches(&self.query) {
                        self.rows.push(Row::Song {
                            song: Arc::new(song),
                            entry: None,
                            queue_index: Some(index),
                        });
                    }
                }
            }
            Page::Folders | Page::Settings | Page::Lyrics => {}
            page => {
                let mut selected: Vec<_> = songs
                    .iter()
                    .filter(|s| s.matches(&self.query))
                    .filter(|s| match page {
                        Page::Favorites => s.favorite,
                        Page::MostPlayed => s.play_count > 0,
                        Page::Album(album, artist) => {
                            s.album == *album && s.album_artist == *artist
                        }
                        Page::Artist(artist) => s.artist == *artist,
                        Page::Genre(genre) => {
                            s.genre == *genre || (genre == "Unknown genre" && s.genre.is_empty())
                        }
                        _ => true,
                    })
                    .cloned()
                    .collect();
                match page {
                    Page::Recent => selected.sort_by_key(|s| std::cmp::Reverse((s.added_at, s.id))),
                    Page::MostPlayed => selected.sort_by_key(|s| std::cmp::Reverse(s.play_count)),
                    Page::Album(..) => selected.sort_by_key(|s| {
                        (s.disc.unwrap_or(1), s.track.unwrap_or(0), s.title.clone())
                    }),
                    _ => sort_songs(
                        &mut selected,
                        self.preferences.sort,
                        self.preferences.descending,
                    ),
                }
                self.rows = selected
                    .into_iter()
                    .map(|song| Row::Song {
                        song: Arc::new(song),
                        entry: None,
                        queue_index: None,
                    })
                    .collect();
            }
        }
    }

    fn title(&self) -> String {
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

    fn render_row(
        &mut self,
        index: usize,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(row) = self.rows.get(index).cloned() else {
            return div().into_any_element();
        };
        match row {
            Row::Group {
                title,
                subtitle,
                cover,
                page,
            } => div()
                .id(("group", index))
                .w_full()
                .h(px(64.))
                .px_4()
                .flex()
                .items_center()
                .gap_4()
                .cursor_pointer()
                .hover(|s| s.bg(cx.theme().muted))
                .child(cover_element(cover, 44.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(subtitle),
                        ),
                )
                .child(Icon::new(IconName::ChevronRight))
                .on_click(
                    cx.listener(move |this, _, window, cx| this.navigate(page.clone(), window, cx)),
                )
                .into_any_element(),
            Row::Song {
                song,
                entry,
                queue_index,
            } => {
                let menu_song = song.clone();
                let id = song.id;
                let playing = if let Some(qindex) = queue_index {
                    self.queue.current_index() == Some(qindex)
                } else {
                    self.queue.current().is_some_and(|s| s.id == id)
                };
                let mut row = div()
                    .id(("track", index))
                    .w_full()
                    .h(px(64.))
                    .px_3()
                    .flex()
                    .items_center()
                    .gap_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .hover(|s| s.bg(cx.theme().muted))
                    .when(playing, |s| s.bg(cx.theme().muted));
                row = row
                    .child(
                        Button::new(("play-track", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Play))
                            .tooltip("Play this track")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                window.focus(&this.focus, cx);
                                if let Some(qindex) = queue_index {
                                    this.queue.select(qindex);
                                } else {
                                    let selected = this.rows[..index]
                                        .iter()
                                        .filter(|r| matches!(r, Row::Song { .. }))
                                        .count();
                                    this.queue.replace(this.visible_songs(), selected);
                                }
                                this.load_current();
                                cx.notify();
                            })),
                    )
                    .when(self.preferences.show_artwork, |row| {
                        row.child(cover_element(song.cover.clone(), 40.))
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(song.title.clone()),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} · {}", song.artist, song.album)),
                            ),
                    )
                    .child(
                        div()
                            .w(px(48.))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(duration_label(song.duration)),
                    )
                    .child(
                        Button::new(("favorite", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Heart))
                            .selected(song.favorite)
                            .tooltip(if song.favorite {
                                "Remove favorite"
                            } else {
                                "Favorite"
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.send_library(LibraryCommand::Favorite(id));
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new(("playlist-add", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::ListPlus))
                            .tooltip("Add to playlist")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.overlay = Some(Overlay::AddToPlaylist(id));
                                cx.notify();
                            })),
                    );
                if let Some(qindex) = queue_index {
                    row = row.child(
                        Button::new(("queue-remove", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::X))
                            .tooltip("Remove from queue")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let was_playing = this.is_playing || this.loading;
                                let current_removed = this.queue.remove(qindex);
                                if current_removed {
                                    if this.queue.is_empty() || !was_playing {
                                        this.stop();
                                    } else {
                                        this.load_current();
                                    }
                                }
                                this.persist_queue();
                                this.rebuild();
                                cx.notify();
                            })),
                    );
                } else if let (Page::Playlist(playlist), Some(entry)) = (self.page.clone(), entry) {
                    row = row
                        .child(
                            Button::new(("up", index))
                                .ghost()
                                .small()
                                .icon(Icon::new(IconName::ChevronUp))
                                .tooltip("Move up")
                                .on_click(cx.listener(move |this, _, _, _| {
                                    this.send_library(LibraryCommand::MoveEntry(
                                        playlist, entry, -1,
                                    ))
                                })),
                        )
                        .child(
                            Button::new(("down", index))
                                .ghost()
                                .small()
                                .icon(Icon::new(IconName::ChevronDown))
                                .tooltip("Move down")
                                .on_click(cx.listener(move |this, _, _, _| {
                                    this.send_library(LibraryCommand::MoveEntry(playlist, entry, 1))
                                })),
                        )
                        .child(
                            Button::new(("remove", index))
                                .ghost()
                                .small()
                                .icon(Icon::new(IconName::X))
                                .tooltip("Remove this playlist entry")
                                .on_click(cx.listener(move |this, _, _, _| {
                                    this.send_library(LibraryCommand::RemoveEntry(entry))
                                })),
                        );
                } else {
                    row = row.child(
                        Button::new(("queue-add", index))
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Plus))
                            .tooltip("Add to queue")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.queue.append((*song).clone());
                                this.persist_queue();
                                this.status = "Added to queue.".into();
                                cx.notify();
                            })),
                    );
                }
                if let Some(qindex) = queue_index {
                    for (label, delta, icon) in [
                        ("queue-up", -1, IconName::ChevronUp),
                        ("queue-down", 1, IconName::ChevronDown),
                    ] {
                        row = row.child(
                            Button::new((label, index))
                                .ghost()
                                .small()
                                .icon(Icon::new(icon))
                                .disabled(
                                    (delta < 0 && qindex == 0)
                                        || (delta > 0 && qindex + 1 >= self.queue.len()),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.queue
                                        .move_to(qindex, qindex.saturating_add_signed(delta));
                                    this.persist_queue();
                                    this.rebuild();
                                    cx.notify();
                                })),
                        );
                    }
                }
                let playlist = if let (Page::Playlist(pl), Some(entry)) = (&self.page, entry) {
                    Some((*pl, entry))
                } else {
                    None
                };
                if queue_index.is_some() || playlist.is_some() {
                    let drag = RowDrag {
                        queue_index,
                        playlist,
                        title: menu_song.title.clone(),
                    };
                    row = row
                        .on_drag(drag, |info: &RowDrag, _, _, cx| cx.new(|_| info.clone()))
                        .on_drop(cx.listener(move |this, drag: &RowDrag, _, cx| {
                            if let (Some(from), Some(to)) = (drag.queue_index, queue_index) {
                                this.queue.move_to(from, to);
                                this.persist_queue();
                                this.rebuild();
                            } else if let (Some((pl, entry)), Some((target_pl, target_entry))) =
                                (drag.playlist, playlist)
                                && pl == target_pl
                                && let Some(playlist) =
                                    this.snapshot.playlists.iter().find(|p| p.id == pl)
                            {
                                let from = playlist.entries.iter().position(|e| e.id == entry);
                                let to = playlist.entries.iter().position(|e| e.id == target_entry);
                                if let (Some(from), Some(to)) = (from, to) {
                                    this.send_library(LibraryCommand::MoveEntry(
                                        pl,
                                        entry,
                                        to as isize - from as isize,
                                    ));
                                }
                            }
                            cx.notify();
                        }));
                }
                let view = cx.entity().downgrade();
                let build_menu =
                    move |mut menu: PopupMenu, _: &mut Window, _: &mut Context<PopupMenu>| {
                        for label in [
                            "Play next",
                            "Add to queue",
                            "Favorite / unfavorite",
                            "Add to playlist",
                            "Go to album",
                            "Go to artist",
                            "Track details",
                        ] {
                            let view = view.clone();
                            let song = menu_song.clone();
                            menu = menu.item(PopupMenuItem::new(label).on_click(
                                move |_, window, cx| {
                                    let _ = view.update(cx, |this, cx| {
                                        match label {
                                            "Play next" => {
                                                this.queue.play_next((*song).clone());
                                                this.persist_queue();
                                                this.status = "Added to play next.".into();
                                                this.rebuild();
                                            }
                                            "Add to queue" => {
                                                this.queue.append((*song).clone());
                                                this.persist_queue();
                                                this.rebuild();
                                            }
                                            "Favorite / unfavorite" => {
                                                this.send_library(LibraryCommand::Favorite(song.id))
                                            }
                                            "Add to playlist" => {
                                                this.overlay = Some(Overlay::AddToPlaylist(song.id))
                                            }
                                            "Go to album" => this.navigate(
                                                Page::Album(
                                                    song.album.clone(),
                                                    song.album_artist.clone(),
                                                ),
                                                window,
                                                cx,
                                            ),
                                            "Go to artist" => this.navigate(
                                                Page::Artist(song.artist.clone()),
                                                window,
                                                cx,
                                            ),
                                            _ => {
                                                this.overlay = Some(Overlay::Details(Box::new(
                                                    (*song).clone(),
                                                )))
                                            }
                                        }
                                        cx.notify();
                                    });
                                },
                            ));
                        }
                        menu
                    };
                row.child(
                    Button::new(("track-menu", index))
                        .ghost()
                        .small()
                        .label("More")
                        .dropdown_menu(build_menu.clone()),
                )
                .context_menu(build_menu)
                .into_any_element()
            }
        }
    }

    fn stop(&mut self) {
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

    fn apply_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        Theme::change(
            if self.preferences.light_theme {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            },
            Some(window),
            cx,
        );
        let color = match self.preferences.accent.as_str() {
            "Red" => 0xe5484d,
            "Blue" => 0x3b82f6,
            "Purple" => 0xa855f7,
            "Orange" => 0xf97316,
            "Teal" => 0x14b8a6,
            "Green" => 0x22c55e,
            _ => {
                if self.preferences.light_theme {
                    0x202024
                } else {
                    0xf4f4f5
                }
            }
        };
        let foreground = if (self.preferences.accent == "Neutral" && !self.preferences.light_theme)
            || matches!(
                self.preferences.accent.as_str(),
                "Orange" | "Green" | "Teal"
            ) {
            rgb(0x18181b).into()
        } else {
            rgb(0xffffff).into()
        };
        Theme::update(cx, |theme| {
            theme.colors.primary = rgb(color).into();
            theme.colors.primary_foreground = foreground;
            theme.colors.primary_hover = rgb(color).into();
            theme.colors.primary_active = rgb(color).into();
            theme.colors.button_primary = rgb(color).into();
            theme.colors.button_primary_foreground = foreground;
            theme.colors.button_primary_hover = rgb(color).into();
            theme.colors.button_primary_active = rgb(color).into();
            theme.colors.ring = rgb(color).into();
            theme.colors.slider_bar = rgb(color).into();
        });
        cx.notify();
    }

    fn seek_to(&mut self, seconds: f64) {
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

    fn set_volume(&mut self, volume: f32, window: &mut Window, cx: &mut Context<Self>) {
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

    fn mute(&mut self, cx: &mut Context<Self>) {
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

    fn fullscreen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.immersive = !self.immersive;
        self.lyric_index = None;
        window.toggle_fullscreen();
        cx.notify();
    }

    fn render_card(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(Row::Group {
            title,
            subtitle,
            cover,
            page,
        }) = self.rows.get(index).cloned()
        else {
            return div().into_any_element();
        };
        let artwork = if let Page::Playlist(id) = page {
            let covers: Vec<_> = self
                .snapshot
                .playlists
                .iter()
                .find(|pl| pl.id == id)
                .into_iter()
                .flat_map(|pl| &pl.entries)
                .filter_map(|entry| self.snapshot.songs.iter().find(|s| s.id == entry.song_id))
                .take(4)
                .map(|s| s.cover.clone())
                .collect();
            if covers.len() > 1 {
                let mut collage = div()
                    .w_full()
                    .h(px(192.))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .rounded_lg();
                for row in 0..2 {
                    collage = collage.child(div().flex().h(px(96.)).children((0..2).map(|col| {
                        div().flex_1().overflow_hidden().child(cover_element(
                            covers
                                .get(row * 2 + col)
                                .cloned()
                                .unwrap_or_else(|| cover.clone()),
                            96.,
                        ))
                    })));
                }
                collage.into_any_element()
            } else {
                cover_element(cover, 192.)
            }
        } else {
            cover_element(cover, 192.)
        };
        div()
            .id(("card", index))
            .flex_1()
            .min_w_0()
            .p_3()
            .rounded_lg()
            .cursor_pointer()
            .hover(|d| d.bg(cx.theme().muted))
            .child(artwork)
            .child(
                div()
                    .mt_3()
                    .truncate()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .child(
                div()
                    .mt_1()
                    .truncate()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(subtitle),
            )
            .on_click(
                cx.listener(move |this, _, window, cx| this.navigate(page.clone(), window, cx)),
            )
            .into_any_element()
    }

    fn render_settings(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div()
            .id("settings-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p_6()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Appearance"),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        Button::new("theme-dark")
                            .label("Dark")
                            .selected(!self.preferences.light_theme)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.preferences.light_theme = false;
                                this.apply_theme(window, cx);
                                this.persist_modes();
                            })),
                    )
                    .child(
                        Button::new("theme-light")
                            .label("Light")
                            .selected(self.preferences.light_theme)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.preferences.light_theme = true;
                                this.apply_theme(window, cx);
                                this.persist_modes();
                            })),
                    ),
            );
        let mut accents = div().flex().flex_wrap().gap_2();
        for name in [
            "Neutral", "Red", "Blue", "Purple", "Orange", "Teal", "Green",
        ] {
            accents = accents.child(
                Button::new(name)
                    .label(name)
                    .selected(self.preferences.accent == name)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.preferences.accent = name.into();
                        this.apply_theme(window, cx);
                        this.persist_modes();
                    })),
            );
        }
        body=body.child(div().child("Accent color").child(accents))
            .child(Switch::new("artwork-toggle").label("Show artwork in song lists").checked(self.preferences.show_artwork).on_click(cx.listener(|this,checked:&bool,_,cx| {
                this.preferences.show_artwork= *checked;this.persist_modes();cx.notify();
            })))
            .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child("Playback & library"))
            .child(Switch::new("watch-toggle").label("Automatically update music folders").checked(self.preferences.watch_folders).on_click(cx.listener(|this,checked:&bool,_,cx| {
                this.preferences.watch_folders= *checked;this.persist_modes();cx.notify();
            })))
            .child(Switch::new("replaygain-toggle").label("Normalize tagged tracks with ReplayGain").checked(self.preferences.replay_gain).on_click(cx.listener(|this,checked:&bool,_,cx| {
                this.preferences.replay_gain= *checked;this.persist_modes();
                this.status="ReplayGain setting applies when the next track starts.".into();cx.notify();
            })))
            .child(div().text_color(cx.theme().muted_foreground).child("Queue and playback position are saved. Playback stays paused when you reopen the app. ReplayGain uses embedded track gain and peak tags."))
            .child(Button::new("clean-covers").label("Clean unused artwork").disabled(self.scanning).on_click(cx.listener(|this,_,_,cx| {
                this.send_library(LibraryCommand::CleanArtwork);this.status="Unused artwork cleanup requested.".into();cx.notify();
            })))
            .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child("Keyboard shortcuts"))
            .child("Space: play/pause · Left/Right: previous/next · Shift+Left/Right: seek 10 seconds")
            .child("Up/Down: volume · Ctrl+M: mute · Ctrl+K or /: search · Ctrl+Q: queue")
            .child("Ctrl+L: lyrics · Ctrl+F or F11: fullscreen player · Alt+Left: back · Escape: close")
            .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child("Reset library"))
            .child(div().text_color(cx.theme().muted_foreground).child("Reset removes library records, playlists, favorites and play counts. Your music files are never deleted."))
            .child(div().flex().gap_3()
                .child(Button::new("reset-keep").danger().label("Reset, keep folders").disabled(self.scanning).on_click(cx.listener(|this,_,_,cx| {this.overlay=Some(Overlay::Reset(false));cx.notify();})))
                .child(Button::new("reset-all").danger().label("Reset everything").disabled(self.scanning).on_click(cx.listener(|this,_,_,cx| {this.overlay=Some(Overlay::Reset(true));cx.notify();}))));
        body.into_any_element()
    }

    fn render_lyrics(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let text = self
            .queue
            .current()
            .map(|s| s.details.lyrics.clone())
            .unwrap_or_default();
        if text.is_empty() {
            return div().flex_1().flex().items_center().justify_center().p_8().text_color(cx.theme().muted_foreground)
                .child("No lyrics for this track. Add embedded lyrics or a matching .lrc file beside the song.").into_any_element();
        }
        let lines = self.lyric_lines.clone();
        if lines.is_empty() {
            return div()
                .id("plain-lyrics")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p_8()
                .text_lg()
                .children(text.lines().map(|line| div().py_2().child(line.to_owned())))
                .into_any_element();
        }
        let active = active_line(&lines, self.position);
        let this = cx.entity().downgrade();
        uniform_list("synced-lyrics", lines.len(), move |range, _, cx| {
            this.update(cx, |_this, cx| {
                range
                    .map(|i| {
                        let line = &lines[i];
                        let seconds = line.seconds;
                        div()
                            .id(("lyric", i))
                            .h(px(72.))
                            .px_8()
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .text_xl()
                            .text_color(if active == Some(i) {
                                cx.theme().primary
                            } else {
                                cx.theme().muted_foreground
                            })
                            .when(active == Some(i), |d| d.font_weight(FontWeight::BOLD))
                            .child(if line.text.is_empty() {
                                "♪".to_owned()
                            } else {
                                line.text.clone()
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.seek_to(seconds);
                                cx.notify();
                            }))
                            .into_any_element()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
        })
        .track_scroll(&self.lyrics_scroll)
        .flex_1()
        .min_h_0()
        .into_any_element()
    }

    fn render_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let mut sidebar = div()
            .id("sidebar")
            .overflow_y_scroll()
            .w(px(220.))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .gap_1()
            .p_4()
            .bg(cx.theme().sidebar)
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .mb_1()
                    .child("Tremors Music"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .mb_4()
                    .child("YOUR MUSIC. YOUR SPACE."),
            );
        for (label, page, icon) in [
            ("Search", Page::Search, IconName::Search),
            ("Songs", Page::Library, IconName::Music),
            ("Albums", Page::Albums, IconName::Disc3),
            ("Artists", Page::Artists, IconName::Users),
            ("Genres", Page::Genres, IconName::Disc3),
            ("Playlists", Page::Playlists, IconName::ListMusic),
            ("Favorites", Page::Favorites, IconName::Heart),
            ("Recently added", Page::Recent, IconName::Clock),
            ("Most played", Page::MostPlayed, IconName::TrendingUp),
            ("Queue", Page::Queue, IconName::ListMusic),
            ("Music folders", Page::Folders, IconName::Folder),
            ("Settings", Page::Settings, IconName::Settings),
        ] {
            sidebar = sidebar.child(
                div()
                    .id(label)
                    .w_full()
                    .h(px(30.))
                    .px_3()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .when(self.page == page, |d| d.bg(cx.theme().muted))
                    .hover(|d| d.bg(cx.theme().muted))
                    .child(Icon::new(icon).size_4())
                    .child(label)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate(page.clone(), window, cx)
                    })),
            );
        }
        sidebar = sidebar.child(
            div()
                .mt_4()
                .mb_2()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(cx.theme().muted_foreground)
                .child("PLAYLISTS"),
        );
        let mut playlists = div()
            .id("sidebar-playlists")
            .min_h(px(80.))
            .flex()
            .flex_col()
            .gap_1();
        for pl in &self.snapshot.playlists {
            let id = pl.id;
            playlists = playlists.child(
                div()
                    .id(("sidebar-playlist", id as usize))
                    .w_full()
                    .h(px(30.))
                    .px_3()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .when(self.page == Page::Playlist(id), |d| d.bg(cx.theme().muted))
                    .hover(|d| d.bg(cx.theme().muted))
                    .child(Icon::new(IconName::ListMusic).size_4())
                    .child(pl.name.clone())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate(Page::Playlist(id), window, cx)
                    })),
            );
        }
        sidebar = sidebar.child(playlists);
        if !matches!(self.page, Page::Playlist(_)) {
            sidebar = sidebar
                .child(Input::new(&self.playlist_name).small())
                .child(
                    Button::new("new-playlist")
                        .ghost()
                        .label("Create playlist")
                        .icon(Icon::new(IconName::Plus))
                        .w_full()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.save_playlist(window, cx)),
                        ),
                );
        }
        let _ = window;
        sidebar
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .mt_3()
                    .child(format!(
                        "v{} · {} songs",
                        env!("CARGO_PKG_VERSION"),
                        self.snapshot.songs.len()
                    )),
            )
            .into_any_element()
    }

    fn render_folders(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let mut body=div().id("folders-content").flex_1().min_h_0().overflow_y_scroll().flex().flex_col().p_6().gap_4()
            .child(div().text_color(cx.theme().muted_foreground).child("Choose folders containing MP3, FLAC, WAV, Ogg Vorbis, or AAC/ALAC in M4A. Your music files stay untouched."))
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
                    .child("Scan warnings"),
            );
            for warning in &self.warnings {
                body = body.child(div().text_sm().child(warning.clone()));
            }
        }
        body.into_any_element()
    }

    fn render_overlay(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
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

    fn render_player(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let current = self.queue.current();
        let cover = current.and_then(|s| s.cover.clone());
        let title = current
            .map(|s| s.title.clone())
            .unwrap_or_else(|| "Nothing playing".into());
        let artist = current
            .map(|s| s.artist.clone())
            .unwrap_or_else(|| "Choose a song from your library".into());
        let disabled = current.is_none();
        div()
            .flex_shrink_0()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .p_4()
            .flex()
            .items_center()
            .gap_5()
            .child(
                div()
                    .w(px(280.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(cover_element(cover, 52.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(artist),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .justify_center()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("shuffle")
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(IconName::Shuffle))
                                    .selected(self.queue.shuffle)
                                    .tooltip("Shuffle")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.queue.set_shuffle(!this.queue.shuffle);
                                        this.persist_modes();
                                        this.rebuild();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("previous")
                                    .ghost()
                                    .icon(Icon::new(IconName::SkipBack))
                                    .disabled(disabled)
                                    .tooltip("Previous / restart")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.previous();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("play-pause")
                                    .primary()
                                    .icon(Icon::new(if self.is_playing {
                                        IconName::Pause
                                    } else {
                                        IconName::Play
                                    }))
                                    .disabled(self.loading)
                                    .tooltip(if self.loading {
                                        "Loading audio…"
                                    } else {
                                        "Play / pause (Space)"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle();
                                        this.rebuild();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("next")
                                    .ghost()
                                    .icon(Icon::new(IconName::SkipForward))
                                    .disabled(disabled)
                                    .tooltip("Next")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.next();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("repeat")
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(if self.queue.repeat == RepeatMode::One {
                                        IconName::Repeat1
                                    } else {
                                        IconName::Repeat
                                    }))
                                    .selected(self.queue.repeat != RepeatMode::Off)
                                    .tooltip(format!("Repeat: {:?}", self.queue.repeat))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.queue.repeat = this.queue.repeat.cycle();
                                        this.persist_modes();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(48.))
                                    .text_xs()
                                    .child(duration_label(self.position)),
                            )
                            .child(div().flex_1().child(Slider::new(&self.seek).horizontal()))
                            .child(
                                div()
                                    .w(px(48.))
                                    .text_xs()
                                    .child(duration_label(self.duration)),
                            ),
                    ),
            )
            .child(
                div()
                    .w(px(250.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        Button::new("mute")
                            .ghost()
                            .small()
                            .icon(Icon::new(if self.muted {
                                IconName::VolumeX
                            } else {
                                IconName::Volume2
                            }))
                            .tooltip("Mute · Ctrl+M")
                            .on_click(cx.listener(|this, _, _, cx| this.mute(cx))),
                    )
                    .child(div().flex_1().child(Slider::new(&self.volume).horizontal()))
                    .child(
                        Button::new("show-lyrics")
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Mic))
                            .tooltip("Lyrics · Ctrl+L")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate(Page::Lyrics, window, cx)
                            })),
                    )
                    .child(
                        Button::new("fullscreen-player")
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Maximize))
                            .tooltip("Fullscreen player · F11")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.fullscreen(window, cx)),
                            ),
                    )
                    .child(
                        Button::new("show-queue")
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::ListMusic))
                            .tooltip("Queue")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate(Page::Queue, window, cx)
                            })),
                    ),
            )
            .into_any_element()
    }
}

impl Render for MusicApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut header = div()
            .p_6()
            .pb_3()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        Button::new("back")
                            .ghost()
                            .icon(Icon::new(IconName::ChevronLeft))
                            .disabled(self.history.is_empty())
                            .on_click(cx.listener(|this, _, window, cx| {
                                if let Some(page) = this.history.pop() {
                                    this.page = page;
                                    this.query.clear();
                                    this.search
                                        .update(cx, |input, cx| input.set_value("", window, cx));
                                    this.rebuild();
                                    cx.notify();
                                }
                            })),
                    )
                    .child(div().flex_1().child(Input::new(&self.search))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex_1()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .child(self.title()),
                    )
                    .when(
                        self.rows.iter().any(|row| matches!(row, Row::Song { .. })),
                        |row| {
                            row.child(
                                Button::new("play-all")
                                    .primary()
                                    .label("Play all")
                                    .icon(Icon::new(IconName::Play))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.queue.replace(this.visible_songs(), 0);
                                        this.load_current();
                                        this.rebuild();
                                        cx.notify();
                                    })),
                            )
                        },
                    )
                    .when(
                        matches!(
                            self.page,
                            Page::Library | Page::Favorites | Page::Artist(_) | Page::Genre(_)
                        ),
                        |row| {
                            let view = cx.entity().downgrade();
                            row.child(
                                Button::new("sort")
                                    .ghost()
                                    .label(format!("Sort: {}", self.preferences.sort.label()))
                                    .dropdown_menu(move |mut menu, _, _| {
                                        for sort in SongSort::ALL {
                                            let view = view.clone();
                                            menu = menu.item(
                                                PopupMenuItem::new(sort.label()).on_click(
                                                    move |_, _, cx| {
                                                        let _ = view.update(cx, |this, cx| {
                                                            this.preferences.sort = sort;
                                                            this.persist_modes();
                                                            this.rebuild();
                                                            cx.notify();
                                                        });
                                                    },
                                                ),
                                            );
                                        }
                                        menu
                                    }),
                            )
                            .child(
                                Button::new("sort-direction")
                                    .ghost()
                                    .label(if self.preferences.descending {
                                        "Descending"
                                    } else {
                                        "Ascending"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.preferences.descending = !this.preferences.descending;
                                        this.persist_modes();
                                        this.rebuild();
                                        cx.notify();
                                    })),
                            )
                        },
                    ),
            );
        if let Page::Playlist(id) = self.page {
            header = header.child(
                div()
                    .flex()
                    .gap_3()
                    .child(div().flex_1().child(Input::new(&self.playlist_name)))
                    .child(Button::new("rename-playlist").label("Rename").on_click(
                        cx.listener(|this, _, window, cx| this.save_playlist(window, cx)),
                    ))
                    .child(
                        Button::new("delete-playlist")
                            .ghost()
                            .label("Delete")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.overlay = Some(Overlay::DeletePlaylist(id));
                                cx.notify();
                            })),
                    ),
            );
        }
        if self.page == Page::Queue {
            header = header.child(
                Button::new("clear-queue")
                    .ghost()
                    .label("Clear queue")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.queue.clear();
                        this.stop();
                        this.rebuild();
                        cx.notify();
                    })),
            );
        }
        let content = if self.page == Page::Settings {
            self.render_settings(cx)
        } else if self.page == Page::Lyrics {
            self.render_lyrics(cx)
        } else if self.page == Page::Folders {
            self.render_folders(cx)
        } else if self.rows.is_empty() {
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .p_8()
                .child(Icon::new(IconName::Music).size_12())
                .child(div().text_lg().child(if self.query.is_empty() {
                    "No tracks here yet"
                } else {
                    "No matching tracks"
                }))
                .child(div().text_color(cx.theme().muted_foreground).child(
                    if self.snapshot.songs.is_empty() {
                        "Add a folder to start building your library."
                    } else {
                        "Choose another view or adjust your search."
                    },
                ))
                .when(self.snapshot.songs.is_empty(), |d| {
                    d.child(
                        Button::new("empty-add-folder")
                            .primary()
                            .label("Add music folder")
                            .disabled(self.picking_folder || self.scanning)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.pick_folder();
                                cx.notify();
                            })),
                    )
                })
                .into_any_element()
        } else if matches!(
            self.page,
            Page::Albums | Page::Artists | Page::Genres | Page::Playlists
        ) {
            let view = cx.entity().downgrade();
            uniform_list(
                "gallery",
                self.rows.len().div_ceil(3),
                move |range, _, cx| {
                    view.update(cx, |this, cx| {
                        range
                            .map(|row| {
                                let mut strip = div().w_full().h(px(280.)).px_4().flex().gap_4();
                                for col in 0..3 {
                                    let index = row * 3 + col;
                                    strip = strip.child(if index < this.rows.len() {
                                        this.render_card(index, cx)
                                    } else {
                                        div().flex_1().into_any_element()
                                    });
                                }
                                strip.into_any_element()
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
                },
            )
            .flex_1()
            .min_h_0()
            .w_full()
            .into_any_element()
        } else {
            let view = cx.entity().downgrade();
            uniform_list("music-list", self.rows.len(), move |range, window, cx| {
                view.update(cx, |this, cx| {
                    range
                        .map(|i| this.render_row(i, window, cx))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
            })
            .flex_1()
            .min_h_0()
            .w_full()
            .into_any_element()
        };
        let sidebar = self.render_sidebar(window, cx);
        let player = self.render_player(cx);
        let overlay = self.render_overlay(cx);
        let immersive = if self.immersive {
            let cover = self.queue.current().and_then(|s| s.cover.clone());
            let title = self
                .queue
                .current()
                .map(|s| s.title.clone())
                .unwrap_or_else(|| "Nothing playing".into());
            let artist = self
                .queue
                .current()
                .map(|s| s.artist.clone())
                .unwrap_or_default();
            Some(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .p_8()
                    .gap_8()
                    .child(
                        div()
                            .w(px(420.))
                            .flex()
                            .flex_col()
                            .justify_center()
                            .gap_5()
                            .child(cover_element(cover, 400.))
                            .child(div().text_3xl().font_weight(FontWeight::BOLD).child(title))
                            .child(
                                div()
                                    .text_lg()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(artist),
                            )
                            .child(
                                Button::new("exit-fullscreen")
                                    .ghost()
                                    .label("Back to library")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.fullscreen(window, cx)
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(self.render_lyrics(cx)),
                    )
                    .into_any_element(),
            )
        } else {
            None
        };
        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_sm()
            .track_focus(&self.focus)
            .key_context("Player")
            .on_action(cx.listener(|this, _: &TogglePlayback, _, cx| {
                this.toggle();
                this.rebuild();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NextTrack, _, cx| {
                this.next();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &PreviousTrack, _, cx| {
                this.previous();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &VolumeUp, window, cx| {
                this.set_volume(this.preferences.volume + 0.05, window, cx)
            }))
            .on_action(cx.listener(|this, _: &VolumeDown, window, cx| {
                this.set_volume(this.preferences.volume - 0.05, window, cx)
            }))
            .on_action(cx.listener(|this, _: &Mute, _, cx| this.mute(cx)))
            .on_action(cx.listener(|this, _: &SeekForward, _, cx| {
                this.seek_to(this.position + 10.);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &SeekBackward, _, cx| {
                this.seek_to(this.position - 10.);
                cx.notify();
            }))
            .on_action(
                cx.listener(|this, _: &ShowQueue, window, cx| {
                    this.navigate(Page::Queue, window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &ShowLyrics, window, cx| {
                this.navigate(Page::Lyrics, window, cx)
            }))
            .on_action(cx.listener(|this, _: &Fullscreen, window, cx| this.fullscreen(window, cx)))
            .on_action(cx.listener(|this, _: &Back, window, cx| {
                if let Some(page) = this.history.pop() {
                    this.page = page;
                    this.query.clear();
                    this.search
                        .update(cx, |input, cx| input.set_value("", window, cx));
                    this.rebuild();
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.navigate(Page::Search, window, cx);
                this.search.update(cx, |input, cx| input.focus(window, cx));
            }))
            .on_action(cx.listener(|this, _: &Escape, window, cx| {
                this.overlay = None;
                if this.immersive {
                    this.fullscreen(window, cx);
                }
                window.focus(&this.focus, cx);
                cx.notify();
            }))
            .when(!self.immersive, |root| {
                root.child(
                    div().flex_1().min_h_0().flex().child(sidebar).child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(header)
                            .child(content)
                            .child(
                                div()
                                    .px_6()
                                    .py_2()
                                    .border_t_1()
                                    .border_color(cx.theme().border)
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(self.status.clone()),
                                    )
                                    .when(self.scanning, |d| {
                                        d.child(
                                            Button::new("status-cancel-scan")
                                                .ghost()
                                                .small()
                                                .label("Cancel")
                                                .on_click(cx.listener(|this, _, _, _| {
                                                    this.library
                                                        .cancel
                                                        .store(true, Ordering::Relaxed)
                                                })),
                                        )
                                    })
                                    .when(!self.warnings.is_empty(), |d| {
                                        d.child(
                                            Button::new("show-warnings")
                                                .ghost()
                                                .small()
                                                .label("View warnings")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.navigate(Page::Folders, window, cx)
                                                })),
                                        )
                                    }),
                            ),
                    ),
                )
            })
            .children(immersive)
            .child(player)
            .children(overlay)
    }
}

fn cover_element(cover: Option<PathBuf>, size: f32) -> AnyElement {
    if let Some(path) = cover {
        img(path)
            .size(px(size))
            .object_fit(ObjectFit::Cover)
            .rounded_md()
            .into_any_element()
    } else {
        div()
            .size(px(size))
            .flex_shrink_0()
            .rounded_md()
            .bg(rgb(0x252832))
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(0x8d92a3))
            .child(Icon::new(IconName::Music))
            .into_any_element()
    }
}

pub fn run() -> anyhow::Result<()> {
    let paths = AppPaths::discover()?;
    let startup_error = std::rc::Rc::new(std::cell::RefCell::new(None));
    let window_error = startup_error.clone();
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Dark, None, cx);
            cx.bind_keys([
                KeyBinding::new("space", TogglePlayback, Some("Player && !Input")),
                KeyBinding::new("ctrl-right", NextTrack, Some("Player && !Input")),
                KeyBinding::new("ctrl-left", PreviousTrack, Some("Player && !Input")),
                KeyBinding::new("ctrl-k", FocusSearch, Some("Player")),
                KeyBinding::new("/", FocusSearch, Some("Player && !Input")),
                KeyBinding::new("right", NextTrack, Some("Player && !Input")),
                KeyBinding::new("left", PreviousTrack, Some("Player && !Input")),
                KeyBinding::new("shift-right", SeekForward, Some("Player && !Input")),
                KeyBinding::new("shift-left", SeekBackward, Some("Player && !Input")),
                KeyBinding::new("up", VolumeUp, Some("Player && !Input")),
                KeyBinding::new("down", VolumeDown, Some("Player && !Input")),
                KeyBinding::new("ctrl-m", Mute, Some("Player && !Input")),
                KeyBinding::new("ctrl-q", ShowQueue, Some("Player && !Input")),
                KeyBinding::new("ctrl-l", ShowLyrics, Some("Player && !Input")),
                KeyBinding::new("ctrl-f", Fullscreen, Some("Player && !Input")),
                KeyBinding::new("f11", Fullscreen, Some("Player")),
                KeyBinding::new("alt-left", Back, Some("Player && !Input")),
                KeyBinding::new("escape", Escape, Some("Player")),
            ]);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);
            if let Err(error) = gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(980.), px(640.))),
                    ..Default::default()
                },
                cx,
                move |window, cx| {
                    window.set_window_title(&format!(
                        "Tremors Music · v{}",
                        env!("CARGO_PKG_VERSION")
                    ));
                    cx.new(|cx| MusicApp::new(paths, window, cx))
                },
            ) {
                *window_error.borrow_mut() = Some(error.context("Could not open Tremors Music"));
                cx.quit();
                return;
            }
            cx.activate(true);
        });
    match startup_error.borrow_mut().take() {
        Some(error) => Err(error),
        None => Ok(()),
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

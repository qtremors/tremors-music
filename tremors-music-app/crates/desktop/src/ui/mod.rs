// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Desktop state and entry point, with worker, playback, browsing, and view modules.

mod browsing;
mod file_tasks;
mod metadata;
mod motion;
mod navigation;
mod playback;
mod runtime;
mod theme;
mod timer;
mod views;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, Selectable, Sizable, Theme, ThemeMode,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    menu::{ContextMenuExt, DropdownMenu, PopupMenu, PopupMenuItem},
    scroll::{ScrollableElement, Scrollbar},
    slider::{Slider, SliderEvent, SliderState, SliderValue},
    switch::Switch,
};
use gpui_kit::{prelude::*, *};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering, mpsc},
    time::{Duration, Instant},
};
use tremors_core::{
    browsing::{
        BrowsePage as Page, BrowseRow as Row, HomeSection, artist_albums, build_rows,
        home_sections, song_index, visible_songs,
    },
    lyrics::{active_line, parse_lrc},
    model::{Snapshot, Song, count_label, duration_label},
    playback::{self as audio, AudioCommand, AudioEvent, AudioHandle},
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

#[derive(Clone)]
enum Overlay {
    SleepTimer,
    DiscardEdits(Page, bool),
    CreatePlaylist,
    RenamePlaylist(i64),
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
    metadata: Option<metadata::MetadataEditor>,
    file_tx: mpsc::Sender<(file_tasks::FileTask, Option<PathBuf>)>,
    file_rx: mpsc::Receiver<(file_tasks::FileTask, Option<PathBuf>)>,
    picking_file: bool,
    minimum_duration: Entity<InputState>,
    sleep_timer: Option<tremors_core::sleep_timer::SleepTimer>,
    timer_finish_track: bool,
    timer_fade: bool,
    timer_label: String,
    timer_gain: f32,
    page_revision: usize,
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
    home_sections: Vec<HomeSection>,
    artist_collections: Vec<Row>,
    show_artist_albums: bool,
    gallery_columns: usize,
    table_scroll: UniformListScrollHandle,
    gallery_scroll: UniformListScrollHandle,
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
    media_revision: Option<u64>,
    media_status: (bool, bool, u64),
}

fn cover_element(cover: Option<PathBuf>, size: f32, cx: &App) -> AnyElement {
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
            .bg(cx.theme().muted)
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child(Icon::new(IconName::Music))
            .into_any_element()
    }
}

fn collection_artwork(cover: Option<PathBuf>, round: bool, cx: &App) -> AnyElement {
    let artwork = div()
        .w_full()
        .aspect_ratio(1.)
        .overflow_hidden()
        .bg(cx.theme().muted)
        .when(round, |d| d.rounded_full())
        .when(!round, |d| d.rounded_lg());
    match cover {
        Some(path) => artwork
            .child(
                img(path)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .when(round, |image| image.rounded_full())
                    .when(!round, |image| image.rounded_lg()),
            )
            .into_any_element(),
        None => artwork
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child(
                Icon::new(if round {
                    IconName::Users
                } else {
                    IconName::Music
                })
                .size_12(),
            )
            .into_any_element(),
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

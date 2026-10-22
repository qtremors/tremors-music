# Tremors Music Changelog

## [Unreleased]

- Added native metadata editing for supported text tags, embedded lyrics, and artwork, with format-specific fields, unsaved-change guards, file conflict checks, and validated atomic saves.
- Added a sleep timer with optional track completion and a final fade; UTF-8 M3U/M3U8 import/export preserves relative paths, order, and duplicate entries.
- Added recently played, folder-cover fallback, local artist/playlist artwork overrides, excluded folders, and a minimum-duration filter that preserves library records.
- Added a true-black OLED theme and reduced-motion preference; refined page, dialog, fullscreen, artwork, hover, and pressed interactions.
- Kept metadata reading/writing, playlist file handling, timer logic, worker operations, picker requests, and feature views in separate modules.

- Added a local library Home page with recently added albums, albums ranked by play count, and personal playlists.
- Redesigned the desktop shell with grouped sidebar navigation, a persistent top player, and global library search.
- Added responsive artwork galleries, circular artist cards, collection headers, and Play and Shuffle actions.
- Refined song tables with compact alternating rows, linked artist and album columns, current-track highlights, and menus for track actions.
- Added playlist creation and renaming dialogs, playlist option menus, and page-specific empty states; reorganized appearance and playback settings into sections.
- Added Albums and Songs views within artist pages, including compilation albums, and scrollbars for long collections.
- Preserved transport controls and status feedback in fullscreen; opening another page returns to the library layout.
- Refined light and dark palettes; new libraries use a red accent while saved appearance preferences remain in effect.

## [0.1.0] - 2026-10-10

- **Native Desktop App:** Added a native Rust GPUI application for Windows and Linux.
- **Music Library:** Browse songs, albums, artists, genres, favorites, recently added, and most played, with global search and sorting.
- **Folders:** Scan incrementally, watch for changes, cancel scans, relocate folders, and clean unused artwork without modifying music files.
- **Queues And Playlists:** Save queues and positions, play next, preserve duplicate entries, and reorder tracks with buttons or drag-and-drop. Reopening restores playback paused.
- **Lyrics And Appearance:** Read local plain and timed lyrics, seek by lyric line, use fullscreen playback, and choose light or dark themes and accents.
- **Desktop Playback:** Added ReplayGain with peak limiting, Windows media controls, and Linux MPRIS integration.
- **Fresh Library:** Use `Tremors Music` and `library.sqlite3` for a new local library.
- **Windows Distribution:** Embedded release shaders, reduced executable size, removed an unnecessary ICU import, and added startup error dialogs and local diagnostic logs.
- **Packages:** Added a minimal per-user Windows installer with the app and license/source notices, smaller portable Windows ZIP, native Linux archives, separate matching source downloads with dependency notices, and SHA-256 checksums.
- **Documentation:** Consolidated development, testing, and verification status in root documents, added Linux build and installation instructions for Debian/Ubuntu, Arch, and Fedora, summarized earlier releases, retained a dedicated privacy policy, and included the complete GPL text in `LICENSE.md`.
- **Website:** Added a single music-focused landing page with an OLED black design, credits, FAQs, and live GitHub release information, stars, forks, and download counts.
- **Developer Credits:** Added the Tremors avatar to the website and removed its white bottom edge.
- **Version Labels:** Keep the app's displayed version, Windows executable resources, and package names aligned with Cargo.

The native application uses GNU GPL v3 only.

---

The 0.0.x entries below document source checkpoints reconstructed on 2026-10-20, including platform sources and dependency-lock records distributed across the commits. The complete desktop application and locked dependency graph are assembled at 0.1.0, which retains its original release date above.

## [0.0.9] - 2026-10-20

- **Windows Packaging:** Added per-user installer and portable ZIP tooling and shader compilation scripts.
- **Linux Packaging:** Added native archive packaging and a desktop launcher.
- **Source And Integrity:** Added corresponding-source archives, dependency notices, and SHA-256 checksum generation.
- **Website:** Added the static product site with responsive layouts, FAQs, privacy information, and GitHub release statistics.

## [0.0.8] - 2026-10-20

- **Appearance Settings:** Added controls for light and dark themes, seven accents, and song-list artwork visibility.
- **Library Preferences:** Added folder-watching and ReplayGain options, artwork cleanup, and library reset controls.
- **Desktop Integration:** Added Windows SMTC and Linux MPRIS publication for current-track metadata and playback state.
- **Regression Coverage:** Added checks for saved sessions, folder updates, relocation, embedded lyrics, ReplayGain, and artwork cleanup.

## [0.0.7] - 2026-10-20

- **Player Controls:** Added the complete transport bar, seek and volume sliders, shuffle, repeat, and playback shortcuts.
- **Lyrics Views:** Added plain and timed lyric presentation, active-line highlighting, automatic scrolling, and line seeking.
- **Fullscreen Player:** Added artwork, track information, lyrics, and controls in an immersive layout.

## [0.0.6] - 2026-10-20

- **Library Views:** Added song lists, album and artist galleries, genres, favorites, recently added, and most-played views.
- **Search And Sorting:** Added search and sort controls, grouped search results, and album and artist navigation.
- **Playlists And Queues:** Added sidebar workflows, row menus, duplicate-entry actions, and drag-and-drop reordering.
- **Folder Management:** Added folder views, relocation and removal dialogs, scan diagnostics, and confirmation overlays.

## [0.0.5] - 2026-10-20

- **Desktop Foundation:** Added GPUI window setup, application state, worker-event handling, and navigation source modules.
- **Playback Coordination:** Connected queue selection, playback transitions, saved positions, and parsed lyric caching.
- **Startup Diagnostics:** Added local logging, panic reporting, native error dialogs, and application resources.
- **Input:** Added folder-picker handling and desktop keyboard-action definitions.

## [0.0.4] - 2026-10-20

- **Library Worker:** Added background commands and events for scanning, playlists, preferences, and playback-state persistence.
- **Folder Watching:** Added recursive watches, debounced incremental scans, scan cancellation, and folder relocation commands.
- **Browsing Models:** Added window-independent library projections, search ranking, sorting, and playlist and queue row identities.
- **Worker Recovery:** Added checks for command failures, scan progress, and cancellation followed by another scan.

## [0.0.3] - 2026-10-20

- **Audio Playback:** Added the audio worker, streaming decoding, pause and resume, seeking, volume, and device-error recovery.
- **Queue Management:** Added duplicate-safe ordering, Play next, reordering, shuffle, repeat, and saved-session restoration.
- **Lyric Timing:** Added LRC timestamp parsing, offsets, multiple timestamps, and active-line selection.
- **ReplayGain:** Added track-gain calculation with peak limiting and invalid-tag handling.

## [0.0.2] - 2026-10-20

- **Music Scanning:** Added local metadata extraction, supported-format detection, and embedded artwork caching.
- **Incremental Updates:** Added file and lyric-sidecar fingerprints to skip unchanged tracks while preserving existing user data.
- **Scan Safety:** Added cancellation, traversal warnings, and safeguards against pruning records after incomplete scans.
- **Format Coverage:** Added original audio fixtures and checks for decoding, seeking, metadata, and reusable artwork.

## [0.0.1] - 2026-10-20

- **Rust Foundation:** Established the Rust 2024 workspace, pinned toolchain, shared models, and desktop scaffold.
- **Local Library:** Added SQLite storage, local data paths, preferences, favorites, play counts, and playlist persistence.
- **Data Integrity:** Preserved duplicate playlist entries and added persistence regression tests.

## Earlier Releases

Tremors Music began as a FastAPI and React application, was adapted into a Tauri desktop application, and was later rewritten in Rust with GPUI. The Rust edition restarts versioning at 0.1.0. The milestones below belong to the earlier web and Tauri editions.

Earlier releases introduced local scanning and metadata, multi-format playback, shuffle and repeat, smart playlists, synced local lyrics, and a fullscreen player. Later updates improved album and artist views, context menus, lazy artwork loading, and playback stability.

The desktop edition added Tauri packaging with a bundled Python backend, custom branding, and Windows installers. Follow-up updates improved scan cancellation, queue synchronization, lyrics rendering, background resource use, accessibility, startup reliability, and security.

- **1.0.0 - FastAPI And React:** Established local library scanning, metadata, playback controls, browsing, playlists, timed lyrics, themes, and fullscreen playback.
- **1.5.0 - Library And Lyrics Improvements:** Improved album and artist context menus, lazy artwork loading, local-only lyrics, and fullscreen playback stability.
- **2.0.0 - Tauri Desktop Edition:** Added a Tauri shell around the React interface and bundled Python backend, with Windows installers and application branding.
- **2.0.1 - Playback And Desktop Reliability:** Improved background resource use and lyric rendering; corrected backend startup, scan cancellation, lyric loading feedback, and shuffled queue synchronization; tightened local API and desktop security settings.
- **2.0.2 - Library Management And Polish:** Added folder path editing and unused artwork cleanup; improved scanner polling, lyric detection, playback state consistency, notifications, and accessibility labels.

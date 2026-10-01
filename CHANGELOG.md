# Tremors Music Changelog

These balanced source checkpoints were reconstructed on 2026-10-20.
The complete desktop application and locked dependency graph are assembled at 0.1.0.

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

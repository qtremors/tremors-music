# Tremors Music Changelog

## [0.1.0] - 2026-10-10

- **Native Desktop App:** Added a native Rust GPUI application for Windows and Linux.
- **Music Library:** Browse songs, albums, artists, genres, favorites, recently added, and most played, with global search and sorting.
- **Folders:** Scan incrementally, watch for changes, cancel scans, relocate folders, and clean unused artwork without modifying music files.
- **Queues And Playlists:** Save queues and positions, play next, preserve duplicate entries, and reorder tracks with buttons or drag-and-drop. Reopening restores playback paused.
- **Lyrics And Appearance:** Read local plain and timed lyrics, seek by lyric line, use fullscreen playback, and choose light or dark themes and accents.
- **Desktop Playback:** Added ReplayGain with peak limiting, Windows media controls, and Linux MPRIS integration.
- **Fresh Library:** Use `Tremors Music` and `library.sqlite3` for a new local library.
- **Windows Distribution:** Embedded release shaders, reduced executable size, removed an unnecessary ICU import, and added startup error dialogs and local diagnostic logs.
- **Packages:** Added a per-user Windows installer, smaller portable Windows ZIP, native Linux archives, separate matching source downloads with dependency notices, and SHA-256 checksums.
- **Documentation:** Consolidated development, testing, and verification status in root documents, added Linux build and installation instructions for Debian/Ubuntu, Arch, and Fedora, summarized earlier releases, retained a dedicated privacy policy, and included the complete GPL text in `LICENSE.md`.
- **Website:** Added a single music-focused landing page with an OLED black design, credits, FAQs, and live GitHub release information, stars, forks, and download counts.
- **Developer Credits:** Added the Tremors avatar to the website and removed its white bottom edge.
- **Version Labels:** Keep the app's displayed version, Windows executable resources, and package names aligned with Cargo.

The native application uses GNU GPL v3 only.

---

## Earlier Releases

Earlier releases introduced local scanning and metadata, multi-format playback, shuffle and repeat, smart playlists, synced local lyrics, and a fullscreen player. Later updates improved album and artist views, context menus, lazy artwork loading, and playback stability.

The desktop edition added Tauri packaging with a bundled Python backend, custom branding, and Windows installers. Follow-up updates improved scan cancellation, queue synchronization, lyrics rendering, background resource use, accessibility, startup reliability, and security.

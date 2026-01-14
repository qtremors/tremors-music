# Tremors Music Changelog

> **Project:** Tremors Music  
> **Version:** 2.0.1  
> **Last Updated:** 2026-01-14

---

## [2.0.1] - 2025-12-23

### Changed
- **Resource Throttling** - Progress bar updates are paused when the app is minimized, reducing background CPU usage.
- **Efficient Lyrics Rendering** - Decoupled sync logic from the main render loop.
- **Animation Loop** - Switched to `requestAnimationFrame` for smoother visuals and automatic pausing.

### Fixed
- **Sidecar Integration** - Fixed an issue where the Python backend was not starting with the app.
- **Installer Size** - Fixed bundling issue where the external backend binary was missing.

### Security
- **Strict CORS** - Restricted origins to `localhost:5173`, `127.0.0.1:8000`, and Tauri protocols; blocked wildcard access.
- **Content Security Policy** - Enabled strict CSP in `tauri.conf.json` to prevent unauthorized script execution.
- **Exception Safety** - Replaced bare `except:` clauses to prevent swallowing system interrupts.

---

## [2.0.0] - 2025-12-11

### Added
- **Native Desktop App** - Windows installer with Tauri (Rust shell).
- **Python Sidecar** - Backend bundled as standalone executable.
- **Resource Efficiency** - Desktop uses ~70% less memory than the web version.
- **Proper Branding** - Custom icons and NSIS installer.
- **New Build Outputs** - `Tremors Music_2.0.0_x64-setup.exe` and `.msi`.

### Changed
- **Project Structure** - Reorganized into `backend/`, `frontend/`, `tauri/`.
- **Build System** - Consolidated `npm run build` for full installer creation.
- **Documentation** - Comprehensive README, ARCHITECTURE, and CONTRIBUTING guides.

---

## [1.5.0] - 2025-12-09

### Added
- **Context Menus** - Added 3-dot menus on Artist and Album detail pages.
- **Sidebar Component** - Refactored for better maintainability.
- **Lazy Loading** - Implemented `loading="lazy"` for images to improve memory.

### Changed
- **Offline-Only Lyrics** - No external fetching, local files only.
- **Memory Optimization** - Excluded lyrics from main song list API.

### Fixed
- Critical crash in Full Screen Player initialization.
- Missing album names in player view.
- Context menu hover triggers.

---

## [1.0.0] - Initial Release

### Added
- **Core Music Player** - Play, pause, seek, shuffle, repeat.
- **Library Management** - Local file scanning and ID3 tag extraction.
- **Multi-format Support** - MP3, FLAC, M4A, WAV, OGG, WMA, AAC.
- **Smart Playlists** - Favorites, Recently Added, Most Played.
- **Synced Lyrics** - Time-synchronized LRC format display.
- **Glassmorphism UI** - Apple Music-inspired design.
- **Full-screen Player** - Immersive view with large artwork.

---

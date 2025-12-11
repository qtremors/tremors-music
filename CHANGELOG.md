# Changelog

All notable changes to Tremors Music are documented in this file.

---

## [2.0.0] - 2025-12-11

### 🎉 Major Release: Native Desktop App

This release introduces a native desktop application using Tauri, delivering significant performance improvements while maintaining the same beautiful interface.

### ✨ Added
- **Native Desktop App** - Windows installer with Tauri (Rust shell)
- **Python Sidecar** - Backend bundled as standalone executable
- **70% Less Memory** - Desktop uses ~100-200MB vs ~900-1400MB for web
- **Proper Branding** - Custom icons, NSIS installer

### 🔧 Changed
- **Project Structure** - Reorganized into `backend/`, `frontend/`, `tauri/`
- **Build System** - Single `npm run build` creates Windows installer
- **Documentation** - Comprehensive README, ARCHITECTURE, CONTRIBUTING

### 📦 New Build Outputs
- `Tremors Music_2.0.0_x64-setup.exe` (NSIS installer)
- `Tremors Music_2.0.0_x64_en-US.msi` (MSI installer)

---

## [1.5.0] - 2025-12-09

### Added
- **Context Menus** - 3-dot menus on Artist and Album detail pages
- **Sidebar Component** - Refactored for better maintainability
- **Lazy Loading** - `loading="lazy"` for images to improve memory

### Changed
- **Offline-Only Lyrics** - No external fetching, local files only
- **Memory Optimization** - Excluded lyrics from main song list API

### Fixed
- Critical crash in Full Screen Player initialization
- Missing album names in player view
- Context menu hover triggers

---

## [1.0.0] - Initial Release

### Added
- **Core Music Player** - Play, pause, seek, shuffle, repeat
- **Library Management** - Local file scanning, ID3 tag extraction
- **Multi-format Support** - MP3, FLAC, M4A, WAV, OGG, WMA, AAC
- **Smart Playlists** - Favorites, Recently Added, Most Played
- **Synced Lyrics** - Time-synchronized LRC format display
- **Glassmorphism UI** - Apple Music-inspired design
- **Full-screen Player** - Immersive view with large artwork

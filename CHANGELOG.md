# Changelog

All notable changes to Tremors Music are documented in this file.

---

## [2.0.0] - 2025-12-11

### 🎉 Unified Monorepo Release

This release consolidates the web and desktop versions into a single monorepo, combining the best features and optimizations from both.

### ✨ Added
- **Unified Codebase** - Single repository powers both web and desktop versions
- **Tauri Desktop App** - Native Windows app with 70% less memory usage than web
- **Proper Branding** - Custom icons, installer with Tremors Music branding

### 🔧 Changed
- **Project Structure** - Reorganized into `backend/`, `frontend/`, and `tauri/` folders
- **Build System** - Single `npm run build` command creates full installer
- **Documentation** - Combined and updated README, ARCHITECTURE, CONTRIBUTING

### 📦 Build Outputs
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

## [1.0.0 Beta] - 2025-12-08

### 🎉 First Desktop Release

#### 🎵 Core Features
- Full playback controls (play, pause, seek, volume)
- Queue management with drag-and-drop
- Shuffle & Repeat modes
- Now Playing full-screen view

#### 📚 Library
- Smart scanning with ID3 tag extraction
- Multi-format support (MP3, FLAC, M4A, WAV, OGG, WMA, AAC)
- Instant search across library
- Persistent sort preferences

#### 🎨 Browse Views
- Songs (virtualized, 5000+ supported)
- Albums (grid with cover art)
- Artists (aggregated from albums)
- Genres (colorful cards)

#### 📝 Playlists
- Custom playlists with drag-to-reorder
- Smart playlists: Favorites, Recently Added, Most Played

#### 🎤 Lyrics
- Synced LRC lyrics with line highlighting
- Extracted from embedded tags
- Database caching for instant access

#### 🎨 UI/UX
- Dark/Light mode
- 6 accent color options
- Apple Music-inspired glassmorphism
- Context menus throughout

---

## [1.0.0] - Initial Web Release

### Added
- Core music player with playback controls
- Library scanning and ID3 tag extraction
- Songs, Albums, Artists, Playlists views
- Glassmorphism UI with Dark/Light mode
- Full Screen Player with large artwork
- Python FastAPI + React frontend

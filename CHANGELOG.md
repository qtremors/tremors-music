# Changelog

All notable changes to Tremors Music will be documented in this file.


## [1.5.0] - 2025-12-09

### Added
- **Context Menus:** Added 3-dot context menus to Artist and Album detail pages for quick actions (Play, Shuffle, Add to Queue).
- **Sidebar Component:** Refactored sidebar into a dedicated component for better maintainability.
- **Lazy Loading:** Implemented `loading="lazy"` for album art and artist images to improve memory usage.
- **Performance:** Optimized `Albums` page load time by moving song counting logic to the backend (SQL subquery) instead of fetching all songs.

### Changed
- **Offline-Only:** Removed all external lyrics fetching. Lyrics are now only sourced from local files or the local database cache.
- **Memory Optimization:** Reduced memory footprint by excluding heavy text fields (lyrics, descriptions) from the main song list API response.
- **UI Polish:**
  - Removed "View Album" placeholder link; album names now only appear if valid.
  - Refined Full Screen Player typography and removed graphical clutter.
  - Fixed visual glitches in the lyrics panel transition.

### Fixed
- **Crashes:** Fixed a critical crash in the Full Screen Player initialization (`setAlbumName`).
- **Data Integrity:** Resolved issues where album names were missing in the player view.
- **UI Bugs:** Fixed missing hover triggers for context menus on detail pages.

## [1.0.0] - Initial Release

### Added
- **Core Music Player:** Play/Pause, Seek, Shuffle, Repeat.
- **Library Management:** Local file scanning, ID3 tag extraction.
- **Organization:** Songs, Albums, Artists, and Playlists views.
- **Visuals:** Modern Glassmorphism UI with Dark/Light mode.
- **Full Screen Mode:** Immersive player with large artwork.
- **Tech Stack:** Python (FastAPI/SQLModel) backend + React (Vite/Tailwind) frontend.

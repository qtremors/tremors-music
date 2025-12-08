# Changelog

All notable changes to Tremors Music are documented in this file.

## [0.9.1] - 2024-12-08

### Added
- **Synced Lyrics Support**: Backend now stores and serves time-synced lyrics (LRC format) in the database.
- **Lyrics Caching**: Results from online searches are now cached locally, making subsequent loads instant.

### Fixed
- **Lyrics Loading Performance**: Rewrote `get_lyrics` endpoint to prioritize: 
  1. Database Cache (Instant)
  2. Local File Embedded Tags (Fast)
  3. Online API (Fallback)
- **Local File Facade**: Scanner now correctly detects and extracts embedded LRC lyrics from music files.

---

## [0.9.0] - 2024-12-08 (Beta Release)

### 🎉 First Beta Release

This marks the first official beta release of Tremors Music, ready for public testing.

### Added
- **Production Logging**: File-based logging system with rotation
  - Logs stored in `logs/tremorsmusic.log`
  - Max 5MB per file, keeps 5 backups
  - Logs startup, errors, and key operations

- **Privacy Documentation**: Added [PRIVACY.md](PRIVACY.md) explaining local-only operation
- **Contribution Guidelines**: Added [CONTRIBUTING.md](CONTRIBUTING.md) for developers
- **License**: Added source-available license

### Changed
- **Process Name**: Backend now shows as `tremorsmusic.exe` in Task Manager (was `backend.exe`)
- **Version Scheme**: Changed to semantic versioning with beta tag
- **Console Window**: Hidden in production (uses log files instead)
- **App Icons**: Generated proper icon set from new logo

### Fixed
- **Favorites Bug**: Now only shows songs you explicitly favorited (rating = 5)
- **Album Art Loading**: Art now appears immediately after scan (no restart needed)
- **Shuffle Algorithm**: Fixed biased shuffle with proper Fisher-Yates implementation
- **Player Store**: `addToQueue` now properly updates `originalQueue`  
- **Player Store**: `clearQueue` now resets all state including `currentSong`
- **DangerZone Toast**: Now visible before page reload
- **Backend Exceptions**: Replaced bare `except` with specific exception types
- **API URLs**: Environment-aware (works in both dev and production)
- **TypeScript Errors**: Fixed `NodeJS.Timeout` and `import.meta.env` issues

### Documentation
- **README.md**: Complete rewrite with production-grade documentation
  - Detailed feature descriptions
  - Installation instructions
  - System impact explanation
  - Troubleshooting guide
  - Future roadmap

---

## [1.4.0] - 2024-12-08

### Added
- **Unified Playlists View**: Smart playlists and user-created playlists consolidated into single view
  - Sidebar shows all playlists together
  - AllPlaylistsPage displays combined playlist listing

- **Visible Favorite Button**: Heart icon now visible on all song displays
  - Shows on hover for unfavorited songs
  - Always visible for favorited songs
  - Consistent behavior across Songs page, Queue, and detail views

### Fixed
- **Play Count Tracking**: Most Played playlist now accurately tracks repeated plays
  - Fixed issue where play counts weren't incrementing on song repeat
  - Proper tracking ensures accurate "Most Played" ordering

---

## [1.3.0] - 2024-12-08

### Added
- **Smart Playlists**: Auto-generated playlists in sidebar
  - Favorites (songs with rating ≥ 4)
  - Recently Added (newest songs)
  - Most Played (top by play count)
  - Heart icon to toggle favorites
  - Play count tracking

- **Queue Improvements**
  - Save Queue as Playlist button
  - Creates playlist with current date name

- **Virtual Scrolling**: Performance boost for large libraries
  - AlbumsPage uses VirtuosoGrid
  - ArtistsPage uses VirtuosoGrid

### Fixed
- **TypeScript Type Safety**
  - Replaced `any` types with proper Album/Artist/Genre interfaces
  - Fixed API function type signatures
  - Added Artist and Genre interfaces to types.ts

---

## [1.2.0] - 2024-12-08

### Added
- **Genres Page**: New genre browsing with colorful gradient cards
  - Sort by name or song count
  - Click to view all songs in a genre
  - Proper handling of multi-genre tags (comma/semicolon separated)
  
- **Filter Persistence**: Sort preferences now persist across page navigation
  - Uses zustand store with localStorage persistence
  - Applies to Songs, Albums, Artists, and Genres pages

- **Album Song Count Sort**: Sort albums by actual number of songs in library

- **Year Column & Sort**: Songs page now shows year column (on xl screens) with clickable sorting

### Fixed
- **Genre URL Encoding**: Genres with special characters (R&B/Soul, Hip Hop/Rap) now work correctly
  - Changed from path parameter to query parameter to avoid `/` issues
  - Backend uses exact genre matching instead of partial `LIKE` matching

- **Multi-Disc Album Track Order**: Albums with multiple discs now sort correctly
  - Sorts by disc_number first, then track_number

- **Context Menu Overflow**: 3-dot menus no longer cause horizontal scrolling
  - Uses fixed positioning with window-aware placement

- **Album Column Display**: Songs page now shows album titles correctly using lookup map

---

## [1.1.0] - 2024-12-07

### Added
- **Artist Detail Enhancements**
  - Clickable album covers navigate to album detail
  - 3-dot context menus on albums (Play, Shuffle, Add to Queue)
  - Integrated AddToPlaylistModal for songs

- **UX Consistency Across Pages**
  - Added SongContextMenu to all song lists
  - Clickable ArtistLink and AlbumLink components for navigation
  - Consistent 3-dot menu patterns

- **Loading Skeletons**: Proper loading states for all pages
  - AlbumGridSkeleton, ArtistGridSkeleton
  - SongListSkeleton, DetailHeaderSkeleton
  - Reduces layout shift during data fetch

### Fixed
- **ESLint & TypeScript Errors**
  - Fixed `useEffect` dependency issues
  - Removed unused variables and imports
  - Fixed components created during render (SortOption extraction)
  - Proper Song type usage across components

- **Library Path Manager**: Fixed useCallback hoisting issue

---

## [1.0.0] - 2024-11-22

### Added
- **Core Music Player**
  - Play, pause, next, previous controls
  - Progress bar with seek functionality
  - Volume control with mute toggle
  - Shuffle and repeat modes (off, all, one)

- **Library Management**
  - Add/remove library paths via Settings
  - Background scanning with real-time progress
  - Metadata extraction (ID3 tags, album art, technical specs)
  - SQLite database for persistent storage

- **Browse Views**
  - Songs: Virtualized list with 5000+ song support
  - Albums: Grid view with cover art
  - Artists: Aggregated from album metadata
  - Playlists: Create, edit, delete custom playlists

- **Player Features**
  - Queue management with drag-to-reorder
  - Now Playing full-screen view
  - Lyrics display (stored in database)
  - Keyboard shortcuts

- **UI/UX**
  - Dark/Light theme toggle
  - Custom accent color picker
  - Apple Music inspired glassmorphism design
  - Responsive layout

- **Search**
  - Smart search across songs, albums, artists
  - Relevance-based scoring
  - Click to play or navigate

### Technical
- React 18 with TypeScript
- Vite for fast development
- TanStack Query for data fetching
- Zustand for state management
- react-virtuoso for virtualized lists
- FastAPI backend with SQLModel ORM
- Mutagen for audio metadata extraction

---

## Development Notes

### Running Locally
```bash
# Backend
cd backend && uv run uvicorn main:app --reload

# Frontend
cd frontend && npm run dev
```

### Project Structure
```
tremors-music/
├── backend/
│   ├── main.py           # FastAPI app entry
│   ├── router/           # API routes
│   ├── models.py         # SQLModel schemas
│   └── scanner.py        # Library scanner
├── frontend/
│   ├── src/
│   │   ├── components/   # Reusable UI components
│   │   ├── pages/        # Route pages
│   │   ├── stores/       # Zustand stores
│   │   ├── lib/          # Utilities & API
│   │   └── types.ts      # TypeScript interfaces
│   └── index.html
└── README.md
```

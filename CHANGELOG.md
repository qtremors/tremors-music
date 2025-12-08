# Changelog

All notable changes to Tremors Music are documented in this file.

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

# Changelog

All notable changes to Tremors Music are documented in this file.

## [1.0.0 Beta] - 2024-12-08

### 🎉 First Public Beta Release

Tremors Music is a beautiful, local-first music player for Windows. This is the first official beta release, ready for public testing.

---

### ✨ Features

#### 🎵 Core Music Player
- **Full Playback Controls** - Play, pause, next, previous, seek, volume
- **Queue Management** - Drag-and-drop reordering, add to queue, save queue as playlist
- **Shuffle & Repeat** - Off, repeat all, repeat one (proper Fisher-Yates shuffle)
- **Now Playing View** - Full-screen immersive experience with large album art
- **Keyboard Shortcuts** - Spacebar (play/pause), arrow keys (skip)

#### 📚 Library Management
- **Smart Scanning** - Reads ID3 tags, album art, and technical metadata
- **Multi-format Support** - MP3, FLAC, M4A, WAV, OGG, WMA, AAC
- **Search** - Instant search across songs, albums, and artists
- **Persistent Sorting** - Your sort preferences are remembered per page

#### 🎨 Browse Views
- **Songs** - Virtualized list supporting 5000+ songs with year column
- **Albums** - Grid view with cover art, sort by title/artist/year/song count
- **Artists** - Aggregated from album metadata with album count sorting
- **Genres** - Colorful gradient cards with song counts
- **Multi-Disc Albums** - Proper disc/track ordering

#### 📝 Playlists
- **Custom Playlists** - Create, edit, rename, delete, drag-to-reorder songs
- **Smart Playlists** - Auto-generated:
  - ❤️ Favorites (songs you've hearted)
  - 🆕 Recently Added (newest songs)
  - 🔥 Most Played (by play count)
- **Unified View** - All playlists shown together in sidebar

#### 🎤 Lyrics
- **Synced Lyrics** - Time-synced LRC format with line highlighting
- **Smart Loading** - Database cache → Embedded tags → Online API
- **Caching** - Search results cached locally for instant repeat loads

#### 🎨 UI/UX
- **Dark/Light Mode** - Toggle between themes
- **Custom Accent Colors** - 12 color options
- **Apple Music-inspired Design** - Glassmorphism effects
- **Loading Skeletons** - Smooth loading states (no layout shift)
- **Context Menus** - Right-click and 3-dot menus throughout
- **Clickable Navigation** - Album/Artist names navigate to detail pages
- **Visible Favorite Button** - Heart icon on all song displays

---

### ⚡ Performance
- **Thumbnail Caching** - Album art resized to 300x300 and cached to disk (~90% memory reduction)
- **Virtual Scrolling** - Songs list uses react-virtuoso for smooth scrolling
- **Efficient Image Loading** - Cache-busted URLs force browser to use optimized thumbnails

---

### 🔧 Technical
- **Tauri Shell** (Rust) - Native window management
- **React 18 + TypeScript** - User interface
- **Python + FastAPI** - Backend service
- **SQLite** - Persistent database storage
- **File-based Logging** - Rotated logs (5MB × 5 backups)
- **Portable Installation** - Per-user install, no admin required

---

### 📁 Files Created

```
[Installation Folder]/
├── Tremors Music.exe       # Main application
├── tremorsmusic.exe        # Backend service
├── music.db                # SQLite database
├── logs/                   # Application logs
└── covers/                 # Cached album thumbnails
```

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

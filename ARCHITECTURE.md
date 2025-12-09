# Architecture Guide: Tremors Music

## 1. Overview

Tremors Music is a Local-First, Hybrid Web Application.

It is designed to be high-performance, handling libraries of 5,000+ songs with zero lag by leveraging a Python backend for heavy lifting and a React frontend for a fluid UI.

**Core Philosophy:**

1. **Local First:** All data stays on disk. SQLite is the source of truth.
    
2. **Performance:** Python handles I/O. React handles Presentation. Virtualization handles Layout.
    
3. **Zero Config:** The app should work out of the box by just pointing to a music folder.
    

## 2. Tech Stack

### Backend (The Engine)

- **Language:** Python 3.11+
    
- **Framework:** FastAPI (Async, High Performance)
    
- **Database:** SQLite + SQLModel (ORM)
    
- **Metadata:** `mutagen` (Robust audio metadata handling)
    

### Frontend (The Interface)

- **Framework:** React + Vite + TypeScript
    
- **Styling:** Tailwind CSS (Utility-first)
    
- **State Management:** Zustand (Global Store with Persistence)
    
- **Data Fetching:** TanStack Query (Caching, Loading states)
    
- **Virtualization:** `react-virtuoso` (Handles 50k+ rows effortlessly)
    
- **Animations:** Framer Motion
    

## 3. Directory Structure (Split Monorepo)

This project uses a clean separation of concerns. Frontend and Backend are distinct projects that happen to live in the same repo.

```
tremors-music/
├── backend/                       # --- PYTHON ENVIRONMENT (FastAPI API and Data Layer) ---
│   ├── router/                    # API ROUTERS (Separation of Endpoints)
│   │   ├── library.py             # Logic for Songs List, Albums List, Artists List, and Sorting.
│   │   ├── media.py               # Endpoints for Cover Art extraction and Lyrics fetching.
│   │   ├── playlists.py           # CRUD operations for Playlists (Create, Add, Delete).
│   │   └── stream.py              # Audio streaming endpoint (handles 206 Byte-Range requests).
│   ├── database.py                # Database connection setup (SQLAlchemy/SQLite engine).
│   ├── main.py                    # FastAPI application entry point, CORS, and lifespan configuration.
│   ├── models.py                  # Database schema definitions (Song, Album, Playlist, etc.) using SQLModel.
│   ├── pyproject.toml             # Python dependency list (used by uv).
│   ├── scanner.py                 # Recursive directory scanning and metadata parsing logic.
│   ├── streamer.py                # Python generator function for streaming file chunks.
│   └── uv.lock                    # Dependency lock file (ensures consistent environment).
├── frontend/                      # --- NODE ENVIRONMENT (React UI) ---
│   ├── public/                    # Static assets not processed by Vite
│   │   ├── music.svg              # Placeholder icon
│   │   └── vite.svg               # Vite logo
│   ├── src/                       # Application source code
│   │   ├── assets/                # Assets processed by Vite
│   │   │   └── react.svg
│   │   ├── components/            # Reusable UI components
│   │   │   ├── AddToPlaylistModal.tsx  # Modal to add selected songs to a playlist.
│   │   │   ├── CreatePlaylistModal.tsx # Modal to create a new playlist (replaces ugly prompt).
│   │   │   ├── FullScreenPlayer.tsx    # Immersive 'Now Playing' view (Art, Controls, Lyrics).
│   │   │   ├── Player.tsx              # The persistent mini-player bar and audio controls.
│   │   │   ├── SettingsModal.tsx       # Modal for managing library paths and UI settings.
│   │   │   ├── Sidebar.tsx             # Main navigation sidebar (Library, Playlists).
│   │   │   ├── SongList.tsx            # Virtualized list component for displaying songs.

│   │   │   └── ToastContainer.tsx      # Component for displaying non-blocking notifications.
│   │   ├── lib/                       # Utility functions
│   │   │   ├── api.ts                  # Axios client setup and API fetching functions (React Query integration).
│   │   │   └── utils.ts                # Formatting helpers (time, file size) and className merger (cn).
│   │   ├── pages/                     # Full-screen views mapped to routes
│   │   │   ├── AlbumDetail.tsx         # Page showing tracks within a specific album.
│   │   │   ├── AlbumsPage.tsx          # Grid view of all albums.
│   │   │   ├── AllPlaylistsPage.tsx    # Grid view of all playlists with CRUD controls.
│   │   │   ├── ArtistDetail.tsx        # Page showing all albums/songs for a specific artist.
│   │   │   ├── ArtistsPage.tsx         # Grid view of all artists.
│   │   │   ├── LibraryPage.tsx         # Main library list view (mounts SongList).
│   │   │   ├── PlaylistDetail.tsx      # Page showing songs inside a specific playlist.
│   │   │   ├── SearchPage.tsx          # Page for global search results.
│   │   │   └── SettingsPage.tsx        # Page for managing paths, themes, and reset.
│   │   ├── stores/                    # Zustand Global State
│   │   │   ├── playerStore.ts          # Manages playback state, current song, queue, and persistence.
│   │   │   ├── themeStore.ts           # Manages UI preferences (Dark/Light mode, accent color, art toggle).
│   │   │   └── toastStore.ts           # Manages state for custom toast notifications.
│   │   ├── App.tsx                    # Main router setup and layout shell.
│   │   ├── index.css                  # Global Tailwind directives and custom CSS (e.g., glassmorphism).
│   │   ├── main.tsx                   # React root entry point and QueryClient provider setup.
│   │   └── types.ts                   # TypeScript interfaces matching SQLModel definitions (Song, Album, etc.).
│   ├── eslint.config.js               # Frontend linting rules.
│   ├── index.html                     # Vite's entry HTML file.
│   ├── package-lock.json              # Specific dependency version locking.
│   ├── package.json                   # Node dependencies list (React, Tailwind, Zustand, etc.).
│   ├── postcss.config.js              # PostCSS configuration for Tailwind.
│   ├── tailwind.config.js             # Tailwind configuration (theme, colors, plugins).
│   ├── tsconfig.app.json              # TypeScript configuration for the application source files.
│   ├── tsconfig.json                  # Base TypeScript configuration.
│   ├── tsconfig.node.json             # TypeScript configuration for Node environment files (Vite, etc.).
│   └── vite.config.ts                 # Vite configuration (HMR, React plugins, and API proxy setup).
├── .gitattributes                 # Git configuration for file handling.
├── .gitignore                     # Files/folders to ignore from Git tracking.
├── ARCHITECTURE.md                # Project structure, rules, and technical explanations.
└── README.md                      # Project introduction and quick start guide.
```
    

## 4. Database Schema

The database is normalized to reduce redundancy.

- **Song:** The central unit. Links to `Album`.
    
- **Album:** Stores `title` and `artist`. We group songs by Album ID.
    
- **LibraryPath:** Stores the user's scan folders.
    
- **PlaylistSong:** A link table that allows a song to be in multiple playlists with a specific order.
    

## 5. Future Improvements (Roadmap)

- **File Watcher:** Implement `watchdog` to auto-update the DB when files are added/deleted on disk.
    
- **Equalizer:** Implement a 10-band EQ using the Web Audio API in the frontend.
    
- **Smart Playlists:** SQL-based dynamic playlists (e.g., "Songs > 5 mins").
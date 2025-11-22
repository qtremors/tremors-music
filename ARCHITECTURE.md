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
    
- **Metadata:** `tinytag` (Fastest pure-python tag reader)
    
- **Process Manager:** `uv` (Dependency management) + `honcho` (Process runner)
    

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
├── Procfile                 # Honcho config (Starts Backend & Frontend together)
├── pyproject.toml           # Python dependencies (uv)
├── uv.lock                  # Python lockfile
│
├── backend/                 # --- PYTHON ENVIRONMENT ---
│   ├── .venv/               # Virtual Environment (Do not commit)
│   ├── main.py              # Entry point. Configures CORS & Lifespan.
│   ├── database.py          # SQLite connection & Session generator.
│   ├── models.py            # Database Schema (Songs, Albums, Playlists).
│   ├── scanner.py           # Recursive directory walker & tag parser.
│   ├── music.db             # The Database File (Do not commit).
│   └── router/              # API Endpoints
│       ├── library.py       # Songs, Artists, Search, Reset logic.
│       ├── media.py         # Cover Art extraction & Lyrics fetching.
│       ├── playlists.py     # CRUD for Playlists.
│       └── stream.py        # Audio Byte-Range Streaming.
│
└── frontend/                # --- NODE ENVIRONMENT ---
    ├── vite.config.ts       # Proxy setup (Redirects /api -> localhost:8000)
    ├── tailwind.config.js   # Design System (Colors, Fonts)
    ├── src/
        ├── components/      # Reusable UI Widgets
        │   ├── Player.tsx   # The persistent bottom bar.
        │   ├── SongList.tsx # The Virtualized List (Core Component).
        │   └── ...
        ├── pages/           # Full-Screen Route Views
        │   ├── LibraryPage.tsx
        │   ├── AlbumsPage.tsx
        │   └── ...
        ├── stores/          # Global State
        │   ├── playerStore.ts # Audio logic (Play, Pause, Queue).
        │   └── ...
        ├── lib/             # Helpers
        │   ├── api.ts       # Axios client with type definitions.
        │   └── utils.ts     # Formatters (Time, Classnames).
        └── App.tsx          # Router & Layout Shell.
```

## 4. Critical Development Rules

### ✅ DO:

1. **Use Calculated Heights for Lists:** When using `react-virtuoso`, **never** rely on nested flexbox expansion (`flex-1`). It causes the "0px Height Bug". Always use `absolute inset-0` or `height: calc(100vh - X)` to enforce dimensions.
    
2. **Normalize Paths:** Always use `os.path.normpath()` in Python before saving/comparing file paths to avoid OS-specific duplicates.
    
3. **Use `uv`:** It is significantly faster than pip. Use `uv add package` to install new backend libs.
    

### ❌ DO NOT:

1. **Do Not Commit `music.db`:** This file is local to your machine.
    
2. **Do Not Block the Main Thread:** File scanning must happen in `BackgroundTasks`.
    
3. **Do Not Use `prompt()`:** Use the custom Modal components (`CreatePlaylistModal`) for user input. It breaks immersion.
    

## 5. Database Schema

The database is normalized to reduce redundancy.

- **Song:** The central unit. Links to `Album`.
    
- **Album:** Stores `title` and `artist`. We group songs by Album ID.
    
- **LibraryPath:** Stores the user's scan folders.
    
- **PlaylistSong:** A link table that allows a song to be in multiple playlists with a specific order.
    

## 6. Future Improvements (Roadmap)

- **File Watcher:** Implement `watchdog` to auto-update the DB when files are added/deleted on disk.
    
- **Equalizer:** Implement a 10-band EQ using the Web Audio API in the frontend.
    
- **Smart Playlists:** SQL-based dynamic playlists (e.g., "Songs > 5 mins").
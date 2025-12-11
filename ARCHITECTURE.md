# Architecture Guide: Tremors Music

A high-performance, local-first music player with web and desktop modes.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Tremors Music                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐     ┌───────────────────────────────┐  │
│  │   Tauri Shell   │────▶│      React Frontend           │  │
│  │   (Rust)        │     │      (UI, Player, State)      │  │
│  └─────────────────┘     └───────────────┬───────────────┘  │
│                                          │ HTTP API         │
│                            ┌─────────────▼───────────────┐  │
│                            │      Python Backend         │  │
│                            │      (FastAPI + SQLite)     │  │
│                            └─────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
tremors-music/
├── backend/                # Python FastAPI backend
│   ├── main.py            # App entry point
│   ├── database.py        # SQLite connection
│   ├── models.py          # SQLModel schemas
│   ├── scanner.py         # Library scanner
│   ├── streamer.py        # Audio streaming
│   └── router/            # API endpoints
│       ├── library.py     # Songs, albums, artists
│       ├── media.py       # Cover art, lyrics
│       ├── playlists.py   # Playlist CRUD
│       └── stream.py      # Audio streaming
├── frontend/               # React TypeScript frontend
│   └── src/
│       ├── components/    # UI components
│       ├── pages/         # Route pages
│       ├── stores/        # Zustand state
│       └── lib/           # API, utilities
├── tauri/                  # Tauri desktop wrapper
│   ├── src-tauri/         # Rust shell
│   │   ├── src/           # Rust source
│   │   └── tauri.conf.json
│   └── scripts/           # Build scripts
│       └── build-backend.mjs
└── assets/                 # Logo files
```

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| **Shell** | Tauri (Rust) |
| **Frontend** | React + TypeScript + Vite |
| **Styling** | Tailwind CSS + Framer Motion |
| **State** | Zustand + TanStack Query |
| **Backend** | Python + FastAPI |
| **Database** | SQLite + SQLModel |
| **Metadata** | Mutagen |

---

## API Endpoints

| Route | Purpose |
|-------|---------|
| `GET /library/songs` | Paginated song list |
| `GET /library/albums` | Album list |
| `GET /stream/{song_id}` | Audio streaming |
| `GET /covers/{album_id}` | Album artwork |
| `POST /library/scan` | Trigger library scan |

---

## Build Process

### Development
- Web: Backend + Frontend running separately
- Tauri: Backend + Tauri dev server

### Production Build
1. `build-backend.mjs` runs PyInstaller to create `tremorsmusic.exe`
2. Tauri bundles frontend + sidecar into installer
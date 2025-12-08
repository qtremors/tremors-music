# Tremors Music Player 🎵

**Tremors Music** is a high-performance, local-first music player built for speed, aesthetics, and audiophiles.

It combines the raw power of a **Python backend** (for file system access and metadata parsing) with the fluidity of a **Modern React UI**, designed to look and feel like a native Apple Music experience on your desktop.

## 🚀 Features

### Library & Browsing
- **Instant Library Loading:** Handles 5,000+ songs effortlessly using virtualized lists
- **Local First:** Your music stays on your disk. No cloud uploads, no accounts required
- **Smart Metadata:** Extracts ID3 tags, Cover Art, and Tech Specs (Bitrate, Format, Sample Rate)
- **Browse By:** Songs, Albums, Artists, Genres, or Custom Playlists
- **Persistent Filters:** Sort preferences are remembered across page navigation

### Playback
- **Full Player Controls:** Play, pause, next, previous, seek, volume
- **Queue Management:** Drag-to-reorder, add songs to queue, view upcoming tracks
- **Shuffle & Repeat:** Off, repeat all, repeat one modes
- **Keyboard Shortcuts:** Spacebar to play/pause, arrow keys for skip

### Organization
- **Genre Browsing:** Colorful genre cards with song counts
- **Multi-Disc Support:** Albums with multiple discs display in correct order
- **Custom Playlists:** Create, edit, and manage your own playlists
- **Context Menus:** Right-click or 3-dot menu on any song/album for quick actions

### Visuals
- **Apple Music Inspired:** Glassmorphism design with blur effects
- **Dark/Light Mode:** Toggle between themes
- **Custom Accent Colors:** Pick your favorite color for highlights
- **Now Playing View:** Full-screen immersive experience with large art

### Lyrics
- **Lyrics Engine:** Displays lyrics stored in song metadata
- **Full-Screen Lyrics:** View in the Now Playing screen

## 🛠️ Tech Stack

| Layer | Technologies |
|-------|-------------|
| **Backend** | Python 3.11+, FastAPI, SQLModel (SQLite), Mutagen |
| **Frontend** | React 18, TypeScript, Vite, Tailwind CSS |
| **State** | Zustand (with localStorage persistence) |
| **Data** | TanStack Query for caching & fetching |
| **UI** | Lucide Icons, react-virtuoso, Framer Motion |
| **Tooling** | `uv` (Python), `npm` (Node.js) |

## ⚡ Quick Start

### Prerequisites

- **Node.js** (v18+)
- **Python** (v3.11+)
- **uv** (Install via `pip install uv`)

### Installation

1. **Clone the repository:**
    ```bash
    git clone https://github.com/qtremors/tremors-music.git
    cd tremors-music
    ```

2. **Install Backend Dependencies:**
    ```bash
    cd backend
    uv sync
    cd ..
    ```

3. **Install Frontend Dependencies:**
    ```bash
    cd frontend
    npm install
    cd ..
    ```

### Running the App

Start both servers in separate terminals:

**Terminal 1 (Backend):**
```bash
cd backend
uv run uvicorn main:app --reload
```

**Terminal 2 (Frontend):**
```bash
cd frontend
npm run dev
```

- **Frontend:** http://localhost:5173
- **Backend API:** http://localhost:8000/docs

## 📖 User Guide

1. **Initial Setup:** Open the app and go to **Settings** (gear icon)
2. **Add Music:** Paste the file path to your music folder (e.g., `D:\Music`) and click **+**
3. **Scan:** Click **Rescan Library**. Watch the progress indicator
4. **Play:** Go to "Songs" or "Albums" and start listening!

## 📚 Additional Documentation

- [CHANGELOG.md](./CHANGELOG.md) - Version history and changes

---

Built with 💖 by Tremors
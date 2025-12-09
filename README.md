# Tremors Music Player 🎵

![Version](https://img.shields.io/badge/version-1.5.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

**Tremors Music** is a high-performance, local-first music player built for speed, aesthetics, and audiophiles.

It combines the raw power of a **Python backend** (for file system access and metadata parsing) with the fluidity of a **Modern React UI**, designed to look and feel like a native Apple Music experience on your desktop.

## 🚀 Features

- **Instant Library Loading:** Handles 5,000+ songs effortlessly using Virtualization.
    
- **Local First:** Your music stays on your disk. No cloud uploads, no accounts required.
    
- **Smart Metadata:** Extracts ID3 tags, Cover Art, and Tech Specs (Bitrate, Format, Sample Rate).
    
- **Lyrics Engine:** Automatically extracts synchronized or plain lyrics from local audio files.
    
- **Organization:** Browse by **Songs**, **Albums**, **Artists**, or create **Custom Playlists**.
    
- **Visuals:** Glassmorphism design, Dark/Light mode, and dynamic accent colors.
    
- **Full Screen Mode:** An immersive "Now Playing" view with large art and lyrics.
    

## 🛠️ Tech Stack

- **Backend:** Python 3.11+, FastAPI, SQLModel (SQLite), Mutagen.
    
- **Frontend:** React, TypeScript, Vite, Tailwind CSS, Framer Motion, Zustand.
    
- **Tooling:** `uv` (Python Package Manager).
    

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

You can start both servers manually in separate terminals:

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

1. **Initial Setup:** Open the app and go to **Settings**.
    
2. **Add Music:** Paste the file path to your music folder (e.g., `D:\Music`) and click **+**.
    
3. **Scan:** Click **Rescan Library**. Watch the terminal for progress.
    
4. **Play:** Go to "Songs" or "Albums" and start listening!
    
---

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed history of changes.

## 📄 License

This project is licensed under the MIT License.

---


## 📊 Performance & Memory

Designed for modern hardware, Tremors Music makes extensive use of GPU acceleration for smooth animations.

**Memory Usage (Firefox/Chromium):**
- **Idling/Background:** ~700MB
- **Full Screen Player:** Spikes to ~1400MB during open/transition, stabilizes around ~900MB.
- **Full Screen + Synced Lyrics:** Spikes to ~1400MB, stabilizes around ~1200MB due to heavy text virtualization and animation layers.

---

## 🔮 Roadmap

- **File Watcher:** Auto-update library when files change on disk.
- **Equalizer:** 10-band audio equalizer using Web Audio API.
- **Smart Playlists:** Dynamic SQL-based playlists (e.g. "Genre = Rock" AND "Year > 2000").
- **Mini Player:** Floating picture-in-picture window.
- **Lyrics Editor:** Edit and save synced LRC lyrics directly in the app.

---

Built with 💖 by Tremors
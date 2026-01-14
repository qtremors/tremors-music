<p align="center">
  <img src="assets/tremorsmusic.png" alt="Tremors Music Logo" width="128" height="128">
</p>

<h1 align="center"><a href="https://github.com/qtremors/tremors-music">Tremors Music</a></h1>

<p align="center">
  A beautiful, high-performance, local-first music player
</p>

<p align="center">
  <img src="https://img.shields.io/badge/React-19.2.0-61DAFB?logo=react" alt="React">
  <img src="https://img.shields.io/badge/FastAPI-0.115.0-009688?logo=fastapi" alt="FastAPI">
  <img src="https://img.shields.io/badge/Tauri-2.0.1-24C8DB?logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/License-TSL-red" alt="License">
</p>

> [!NOTE]
> **Personal Project** 🎯 I built this to create a private, high-performance music listening experience that stays completely local. It explores the integration of Python backends with modern React frontends via Tauri sidecars.

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🔒 **Complete Privacy** | Your music stays on your computer. No accounts, no cloud, completely offline. |
| 🎨 **Beautiful UI** | Apple Music-inspired design with glassmorphism and smooth animations. |
| ⚡ **Speed** | High-performance virtualized lists capable of handling 10,000+ songs efficiently. |
| 🎵 **Multi-format** | Supports MP3, FLAC, M4A, WAV, OGG, WMA, AAC, and ALAC. |
| 📝 **Synced Lyrics** | Display embedded synchronized lyrics (LRC format) during playback. |
| 🎛️ **Smart Playlists** | Automatically generated playlists: Favorites, Recently Added, and Most Played. |
| 🖥️ **Desktop App** | Native Windows application powered by Tauri for lower resource impact. |

---

## 💻 System Requirements

### Desktop App (Tauri) - Recommended
| Resource | Usage |
|----------|-------|
| **Disk Space** | ~150 MB |
| **Memory (App)** | ~100-200 MB |
| **CPU** | Minimal (spikes during library scan) |
| **Network** | None (completely offline) |

### Web Version (Browser)
| Resource | Usage |
|----------|-------|
| **Browser Tab** | ~700-900 MB |
| **Full-screen Player** | ~900-1100 MB |
| **With Lyrics Panel** | ~1200-1500 MB |
| **Terminal/IDE** | ~200-500 MB |

> **Note:** The desktop app uses ~70% less memory than the web version.

---

## 🚀 Quick Start

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| **Node.js** | v18+ | Frontend & Tauri Build |
| **Python** | v3.11+ | Backend logic & Metadata extraction |
| **uv** | latest | Fast Python package manager |
| **Rust** | latest | Tauri desktop shell build |

### 1. Clone & Install

```bash
git clone https://github.com/qtremors/tremors-music.git
cd tremors-music

# Install all dependencies
cd backend && uv sync && cd ..
cd frontend && npm install && cd ..
cd tauri && npm install && cd ..
```

### 2. Running the Web Version

```bash
# Terminal 1: Backend
cd backend && uv run uvicorn main:app --reload

# Terminal 2: Frontend
cd frontend && npm run dev
```
Visit **http://localhost:5173**

### 3. Running the Desktop Version (Tauri)

```bash
# Terminal 1: Backend
cd backend && uv run uvicorn main:app --reload

# Terminal 2: Tauri
cd tauri && npm run dev
```

---

## 🛠️ Tech Stack

| Layer | Technology |
|-------|------------|
| **Shell** | Tauri (Rust) |
| **Frontend** | React + TypeScript + Vite |
| **Styling** | Tailwind CSS + Framer Motion |
| **State** | Zustand + TanStack Query |
| **Backend** | Python + FastAPI |
| **Database** | SQLite + SQLModel |
| **Metadata** | Mutagen |

---

## 📁 Project Structure

```
tremors-music/
├── backend/            # Python FastAPI backend
├── frontend/           # React TypeScript frontend
├── tauri/              # Tauri desktop wrapper
│   ├── src-tauri/      # Rust shell & sidecar config
│   └── scripts/        # Build scripts
├── assets/             # Branding & Logo files
├── DEVELOPMENT.md      # Architecture & Developer guide
├── CHANGELOG.md        # Version history
├── LICENSE.md          # License terms
└── README.md
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [DEVELOPMENT.md](DEVELOPMENT.md) | Architecture, technical setup, and API reference |
| [CHANGELOG.md](CHANGELOG.md) | Version history and release notes |
| [LICENSE.md](LICENSE.md) | License terms (Tremors Source License) |
| [PRIVACY.md](PRIVACY.md) | Privacy policy and data handling |

---

## 📄 License

**Tremors Source License (TSL)** - Source-available license allowing viewing, forking, and derivative works with **mandatory attribution**. Commercial use requires written permission.

See [LICENSE.md](LICENSE.md) for full terms.

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/qtremors">Tremors</a>
</p>
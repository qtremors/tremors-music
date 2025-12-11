# Tremors Music 🎵

<p align="center">
  <img src="assets/tremorsmusic.png" alt="Tremors Music Logo" width="128" height="128">
</p>

<p align="center">
  <strong>A beautiful, high-performance, local-first music player</strong>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-development">Development</a> •
  <a href="#-building">Building</a>
</p>

---

## ✨ Features

- 🔒 **Complete Privacy** - Your music stays on your computer. No accounts, no cloud.
- 🎨 **Beautiful UI** - Apple Music-inspired design with glassmorphism
- ⚡ **Speed** - Handles 10,000+ songs with virtualized lists
- 🎵 **Multi-format** - MP3, FLAC, M4A, WAV, OGG, WMA, AAC
- 📝 **Synced Lyrics** - Display embedded synchronized lyrics
- 🎛️ **Smart Playlists** - Favorites, Recently Added, Most Played

---

## � System Impact

### Desktop App (Tauri)
| Resource | Usage |
|----------|-------|
| **Disk Space** | ~150 MB |
| **Memory** | ~100-200 MB during normal use |
| **CPU** | Minimal (spikes during library scan) |
| **Network** | None (completely offline) |

### Web Version (Browser)
| Resource | Usage |
|----------|-------|
| **Idling** | ~700 MB |
| **Full Screen Player** | ~900-1400 MB |
| **With Lyrics** | ~1200 MB |

> The desktop app uses ~70% less memory than the web version thanks to Tauri's lightweight Rust shell.

---

## �🚀 Quick Start

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| **Node.js** | v18+ | Frontend |
| **Python** | v3.11+ | Backend |
| **uv** | latest | Python package manager |
| **Rust** | latest | Tauri (desktop builds only) |

```bash
# Install uv (Python package manager)
pip install uv

# Install Rust (for Tauri desktop builds)
# Visit https://rustup.rs
```

---

## 💻 Development

### 1. Clone & Install Dependencies

```bash
git clone https://github.com/qtremors/tremors-music.git
cd tremors-music

# Backend
cd backend && uv sync && cd ..

# Frontend
cd frontend && npm install && cd ..

# Tauri (for desktop)
cd tauri && npm install && cd ..
```

### 2. Running the Web Version

```bash
# Terminal 1 - Backend
cd backend
uv run uvicorn main:app --reload

# Terminal 2 - Frontend
cd frontend
npm run dev
```

Open **http://localhost:5173**

### 3. Running the Tauri Version (Desktop)

```bash
# Terminal 1 - Backend
cd backend && uv run uvicorn main:app --reload

# Terminal 2 - Tauri
cd tauri && npm run dev
```

---

## 🔨 Building

### Build Desktop Executable

```bash
cd tauri
npm run build
```

Installers created in `tauri/src-tauri/target/release/bundle/`

---

## 📁 Project Structure

```
tremors-music/
├── backend/       # Python FastAPI backend
├── frontend/      # React TypeScript frontend
├── tauri/         # Tauri desktop wrapper
│   ├── src-tauri/ # Rust shell
│   └── scripts/   # Build scripts
└── assets/        # Logo files
```

---

## 📚 Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical details
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guide
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [PRIVACY.md](PRIVACY.md) - Privacy notice

---

## 📄 License

Source-available for viewing and personal use. See [LICENSE](LICENSE).

---

<p align="center">
  Built with 💖 by <a href="https://github.com/qtremors">Tremors</a>
</p>
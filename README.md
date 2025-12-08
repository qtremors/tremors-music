# Tremors Music 🎵

<p align="center">
  <img src="assets/tremorsmusic.png" alt="Tremors Music Logo" width="128" height="128">
</p>

<p align="center">
  <strong>A beautiful, local-first music player for your personal collection</strong>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-how-it-works">How It Works</a> •
  <a href="#-system-impact">System Impact</a> •
  <a href="#-troubleshooting">Troubleshooting</a> •
  <a href="#-roadmap">Roadmap</a>
</p>

---

## ✨ What is Tremors Music?

Tremors Music is a **high-performance desktop music player** designed for users who want:

- 🔒 **Complete Privacy** - Your music stays on your computer. No accounts, no cloud, no tracking.
- 🎨 **Beautiful UI** - Apple Music-inspired design with glassmorphism effects
- ⚡ **Speed** - Handles 10,000+ songs with virtualized lists and smart caching
- 🎯 **Simplicity** - Point it at your music folder and start listening

Perfect for audiophiles, privacy-conscious users, and anyone tired of bloated music apps.

---

## 🚀 Features

### Library Management
| Feature | Description |
|---------|-------------|
| **Smart Scanning** | Automatically reads ID3 tags, album art, and technical metadata |
| **Multi-format Support** | MP3, FLAC, M4A, WAV, OGG, WMA, AAC |
| **Browse By** | Songs, Albums, Artists, Genres |
| **Search** | Instant search across your entire library |
| **Persistent Sorting** | Your sort preferences are remembered |

### Playback
| Feature | Description |
|---------|-------------|
| **Full Controls** | Play, pause, next, previous, seek, volume |
| **Queue Management** | Drag-and-drop reordering, add to queue |
| **Shuffle & Repeat** | Off, repeat all, repeat one modes |
| **Keyboard Shortcuts** | Spacebar (play/pause), arrows (skip) |
| **Now Playing View** | Immersive full-screen experience |

### Organization
| Feature | Description |
|---------|-------------|
| **Custom Playlists** | Create, edit, rename, delete |
| **Smart Playlists** | Favorites, Recently Added, Most Played |
| **Multi-Disc Albums** | Proper disc/track ordering |
| **Genre Browsing** | Colorful cards with song counts |

### Visuals
| Feature | Description |
|---------|-------------|
| **Dark/Light Mode** | Toggle between themes |
| **Custom Accent Colors** | 12 color options |
| **Album Art Display** | Large artwork in Now Playing |
| **Lyrics View** | Display embedded lyrics |

---

## 📥 Installation

### For Users (Recommended)

1. **Download** the latest installer from [Releases](https://github.com/qtremors/tremors-music/releases)
2. **Run** the installer (`Tremors Music_x.x.x_x64-setup.exe`)
3. **Choose** your installation folder
4. **Launch** Tremors Music from the Start Menu or desktop shortcut

### For Developers

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup instructions.

---

## ⚙️ How It Works

Tremors Music uses a **hybrid architecture** for the best of both worlds:

```
┌─────────────────────────────────────────────────────────┐
│                    Tremors Music                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────┐     ┌─────────────────────────┐   │
│  │   Tauri Shell   │────▶│   React Frontend        │   │
│  │   (Window)      │     │   (UI, Player, State)   │   │
│  └─────────────────┘     └───────────┬─────────────┘   │
│                                      │ HTTP API        │
│                          ┌───────────▼─────────────┐   │
│                          │   Python Backend        │   │
│                          │   (tremorsmusic.exe)    │   │
│                          │   - File scanning       │   │
│                          │   - Audio streaming     │   │
│                          │   - Database (SQLite)   │   │
│                          └─────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Components

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Shell** | Tauri (Rust) | Window management, native integration |
| **Frontend** | React + TypeScript | User interface, playback controls |
| **Backend** | Python + FastAPI | File scanning, metadata, streaming |
| **Database** | SQLite | Library metadata, playlists, settings |

---

## 💻 System Impact

### What Gets Installed

When you install Tremors Music, the following is created in your chosen installation folder:

```
[Installation Folder]/
├── Tremors Music.exe       # Main application
├── tremorsmusic.exe        # Backend service (runs alongside main app)
├── music.db                # SQLite database (your library)
├── logs/                   # Application logs
│   └── tremorsmusic.log    # Log file (rotated, max 5MB × 5)
└── covers/                 # Cached album artwork
```

### System Resources

| Resource | Usage |
|----------|-------|
| **Disk Space** | ~150 MB (app) + varies (database/cache) |
| **Memory** | ~100-200 MB during normal use |
| **CPU** | Minimal (spikes during library scan) |
| **Network** | None (completely offline) |

### Processes Running

When Tremors Music is open:
- `Tremors Music.exe` - Main application window
- `tremorsmusic.exe` - Backend service (starts/stops with app)

### Startup Behavior

- ❌ Does NOT start with Windows
- ❌ Does NOT run in background
- ✅ Fully closes when you close the window

---

## 🗑️ Uninstallation

### Complete Removal

1. Open **Settings** > **Apps** > **Tremors Music**
2. Click **Uninstall**
3. All application files are removed

### What's Removed
- All program files
- Your library database
- Cached album art
- Log files

### Keeping Your Data

If you want to preserve your library before uninstalling:
- Copy `music.db` from the installation folder
- After reinstalling, place it back in the same location

---

## 🔧 Troubleshooting

### App Won't Start

1. Check if Windows Defender is blocking it
2. Run as Administrator
3. Check `logs/tremorsmusic.log` for errors

### Songs Not Appearing

1. Ensure your music files have ID3 tags
2. Check the scan completed (no spinner in Settings)
3. Supported formats: MP3, FLAC, M4A, WAV, OGG, WMA, AAC

### Album Art Not Showing

1. Art must be embedded in the audio file
2. Try rescanning the library
3. Check if `covers/` folder has write permissions

### Playback Issues

1. Check the file isn't corrupted
2. Ensure the file path hasn't changed
3. Try removing and re-adding the library path

### Need More Help?

1. Check the [logs/tremorsmusic.log](logs/) file for errors
2. Open an issue on [GitHub](https://github.com/qtremors/tremors-music/issues)

---

## 🛣️ Roadmap

### Coming Soon
- [ ] First-run welcome wizard
- [ ] Auto-update checker (opt-in)
- [ ] Crash reporting (opt-in)
- [ ] Equalizer controls

### Future Ideas
- [ ] Music visualizer
- [ ] Last.fm scrobbling (opt-in)
- [ ] Podcast support
- [ ] macOS and Linux builds
- [ ] Discord Rich Presence

### Included in 1.0.0 Beta
- [x] Full music playback with queue management
- [x] Library scanning with metadata extraction
- [x] Smart & custom playlists
- [x] Synced lyrics support
- [x] Thumbnail caching for performance
- [x] Dark/Light mode with accent colors
- [x] Fisher-Yates shuffle algorithm
- [x] File-based logging

---

## 📚 Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history
- [PRIVACY.md](PRIVACY.md) - Privacy notice
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guide
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical details

---

## 📄 License

This project is source-available for viewing and personal use. See [LICENSE](LICENSE) for details.

**TL;DR:**
- ✅ View source, download and use personally
- ✅ Contribute improvements
- ❌ Redistribute as your own product

---

## 🙏 Acknowledgments

Built with amazing open source technologies:
- [Tauri](https://tauri.app/) - Desktop app framework
- [React](https://react.dev/) - UI framework
- [FastAPI](https://fastapi.tiangolo.com/) - Python API framework
- [Mutagen](https://mutagen.readthedocs.io/) - Audio metadata
- [Lucide](https://lucide.dev/) - Beautiful icons

---

<p align="center">
  Built with 💖 by <a href="https://github.com/qtremors">Tremors</a>
</p>
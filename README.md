# Tremors Music Player 🎵

**Tremors Music** is a high-performance, local-first music player built for speed, aesthetics, and audiophiles.

It combines the raw power of a **Python backend** (for file system access and metadata parsing) with the fluidity of a **Modern React UI**, designed to look and feel like a native Apple Music experience on your desktop.

## 🚀 Features

- **Instant Library Loading:** Handles 5,000+ songs effortlessly using Virtualization.
    
- **Local First:** Your music stays on your disk. No cloud uploads, no accounts required.
    
- **Smart Metadata:** Extracts ID3 tags, Cover Art, and Tech Specs (Bitrate, Format, Sample Rate).
    
- **Lyrics Engine:** Automatically fetches synchronized or plain lyrics from open sources.
    
- **Organization:** Browse by **Songs**, **Albums**, **Artists**, or create **Custom Playlists**.
    
- **Visuals:** Glassmorphism design, Dark/Light mode, and dynamic accent colors.
    
- **Full Screen Mode:** An immersive "Now Playing" view with large art and lyrics.
    

## 🛠️ Tech Stack

- **Backend:** Python 3.11+, FastAPI, SQLModel (SQLite), Tinytag.
    
- **Frontend:** React, TypeScript, Vite, Tailwind CSS, Framer Motion, Zustand.
    
- **Tooling:** `uv` (Python Package Manager).
    

## ⚡ Quick Start

### Prerequisites

- **Node.js** (v18+)
    
- **Python** (v3.11+)
    
- **uv** (Install via `pip install uv`)
    

### Installation

1. **Clone the repository:**
    
    ```
    git clone [https://github.com/yourusername/tremors-music.git](https://github.com/yourusername/tremors-music.git)
    cd tremors-music
    ```
    
2. **Install Backend Dependencies:**
    
    ```
    cd backend
    uv sync
    cd ..
    ```
    
3. **Install Frontend Dependencies:**
    
    ```
    cd frontend
    npm install
    cd ..
    ```
    

### Running the App

You can start both servers manually in separate terminals:

**Terminal 1 (Backend):**

```
cd backend
uv run uvicorn main:app --reload
```

**Terminal 2 (Frontend):**

```
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
    

## 🤝 Contributing

Contributions are welcome! Please check `ARCHITECTURE.md` for detailed structural information and development guidelines.

_Built with ❤️ by Tremors_
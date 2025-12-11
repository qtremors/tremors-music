# Contributing to Tremors Music

## Prerequisites

- **Node.js** v18+
- **Python** v3.11+
- **uv** - Python package manager (`pip install uv`)
- **Rust** - For Tauri builds ([rustup.rs](https://rustup.rs))

---

## Development Setup

```bash
# Clone the repo
git clone https://github.com/qtremors/tremors-music.git
cd tremors-music

# Install backend dependencies
cd backend
uv sync
cd ..

# Install frontend dependencies
cd frontend
npm install
cd ..

# Install Tauri dependencies
cd tauri
npm install
cd ..
```

---

## Running Locally

### Web Version
```bash
# Terminal 1 - Backend
cd backend
uv run uvicorn main:app --reload

# Terminal 2 - Frontend
cd frontend
npm run dev
```

### Tauri Version
```bash
# Terminal 1 - Backend
cd backend
uv run uvicorn main:app --reload

# Terminal 2 - Tauri
cd tauri
npm run dev
```

---

## Building

### Build Desktop Executable
```bash
cd tauri
npm run build
```

Installer created in `tauri/src-tauri/target/release/bundle/`

---

## Code Style

### TypeScript/React
- Use TypeScript strict mode
- Functional components with hooks
- Zustand for state management

### Python
- Follow PEP 8
- Use type hints
- FastAPI patterns

---

## Pull Request Process

1. Create a feature branch
2. Make changes with clear commits
3. Test both web and Tauri versions
4. Submit PR with description

---

Thank you for contributing! 🎶

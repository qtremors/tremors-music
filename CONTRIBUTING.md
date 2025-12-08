# Contributing to Tremors Music

Thank you for your interest in contributing to Tremors Music! 🎵

## Getting Started

### Prerequisites

- **Node.js** v18 or higher
- **Python** v3.11 or higher
- **Rust** (latest stable) - required for Tauri
- **uv** - Python package manager (`pip install uv`)

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/qtremors/tremors-music.git
   cd tremors-music
   ```

2. **Install backend dependencies:**
   ```bash
   cd backend
   uv sync
   cd ..
   ```

3. **Install frontend dependencies:**
   ```bash
   cd frontend
   npm install
   cd ..
   ```

4. **Run in development mode:**
   ```bash
   # Terminal 1 - Backend
   cd backend
   uv run uvicorn main:app --reload

   # Terminal 2 - Frontend
   cd frontend
   npm run dev
   ```

5. **Or run as Tauri app (development):**
   ```bash
   npm run tauri dev
   ```

## Project Structure

```
tremors-music/
├── backend/           # Python FastAPI backend
│   ├── main.py        # Entry point
│   ├── database.py    # SQLite database setup
│   ├── models.py      # SQLModel schemas
│   ├── scanner.py     # Music file scanner
│   └── router/        # API endpoints
├── frontend/          # React TypeScript frontend
│   ├── src/
│   │   ├── components/  # React components
│   │   ├── pages/       # Page components
│   │   ├── stores/      # Zustand state stores
│   │   └── lib/         # Utilities and API
├── src-tauri/         # Tauri configuration and Rust code
└── scripts/           # Build scripts
```

## Code Style

### TypeScript/React
- Use TypeScript strict mode
- Prefer functional components with hooks
- Use Zustand for state management
- Follow the existing component patterns

### Python
- Follow PEP 8 guidelines
- Use type hints where possible
- Use FastAPI patterns for endpoints
- Handle errors gracefully

## Pull Request Process

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** with clear, atomic commits

3. **Test your changes:**
   - Run the app and verify functionality
   - Check for TypeScript errors: `cd frontend && npm run build`
   - Ensure no regressions

4. **Submit a pull request:**
   - Describe what your PR does
   - Reference any related issues
   - Include screenshots for UI changes

## Reporting Issues

When reporting bugs, please include:
- Steps to reproduce
- Expected vs actual behavior
- Your OS and version
- Screenshots if applicable
- Log files from `logs/tremorsmusic.log` (if relevant)

## Feature Requests

We welcome feature ideas! Please:
- Check existing issues first
- Describe the use case
- Explain why it would benefit users

## License

By contributing, you agree that your contributions will be licensed under the project's license terms. See [LICENSE](LICENSE) for details.

---

Thank you for helping make Tremors Music better! 🎶

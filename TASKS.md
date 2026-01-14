# Tremors Music - Tasks

> **Project:** Tremors Music  
> **Version:** 2.0.1  
> **Last Updated:** 2026-01-13

---

---

## ✅ Completed (v2.0.0)

### Infrastructure
- [x] Consolidated web and desktop repos into monorepo
- [x] Fresh Tauri initialization with proper branding
- [x] Build system working (npm run build)
- [x] Memory optimizations from web version
- [x] Custom icons and installer branding
- [x] Updated all core documentation

---

## 🚧 In Progress

### Documentation
- [/] Align project documentation with new Templates
  - [x] Update README.md
  - [x] Create DEVELOPMENT.md (merged ARCHITECTURE/CONTRIBUTING)
  - [x] Align TASKS.md
  - [/] Align CHANGELOG.md
  - [ ] Replace LICENSE with LICENSE.md

---

## 📋 To Do

### High Priority
- [ ] **Queue Shuffle Logic**
  - Fix logic where `addToQueue()` inserts after current song in `queue` but appends to end of `originalQueue`, causing desync when shuffle is toggled.
- [ ] **Type Consistency**
  - Standardize `getPlaylistSongs(id: string)` to use `number` ID parameters to match the rest of the API.

### Medium Priority
- [ ] **Library Path Management**
  - Implement `saveEdit()` in `LibraryPathManager.tsx` to call existing backend `PATCH /library/paths/{id}` endpoint.
- [ ] **UI Polish**
  - Replace `alert()` calls in `QueuePanel.tsx`, `SettingsModal.tsx`, and `LibraryPathManager.tsx` with toast notifications.
  - Add guard against NaN in `Player.tsx` progress bar calculation.

### Low Priority
- [ ] **Accessibility**
  - Add meaningful `alt` attributes to cover images in `FullScreenPlayer.tsx`, `SongList.tsx`, and `SearchPage.tsx`.
- [ ] **Code Quality**
  - Deduplicate Fisher-Yates shuffle implementation between `playerStore.ts` and `utils.ts`.
  - Extract `app_dir` resolution to a shared utility.

---

## 🐛 Bug Fixes

- [ ] **Scanner Control:** `handleStopScan()` in `ScannerControl.tsx` correctly calls backend `/library/scan/stop`.
- [ ] **Lyrics Display:** Fix "Searching for lyrics..." shown indefinitely if fetch fails in `FullScreenPlayer.tsx`.
- [ ] **Scanner Logic:** Refine synced lyrics detection in `scanner.py` using regex instead of simple bracket heuristics.

---

## 💡 Ideas / Future

- [ ] File watcher for auto-updating library on file changes
- [ ] Equalizer controls (Web Audio API)
- [ ] Mini player mode
- [ ] macOS and Linux builds
- [ ] Keyboard shortcuts help modal (? key)
- [ ] Export/import playlists
- [ ] **Technical Debt:** Consider using TanStack Query for scan status polling instead of raw `setInterval`.

---

## 🏗️ Architecture Notes

- **Sidecar Pattern:** Python backend is bundled as an external binary and managed by the Tauri Rust shell.
- **Virtualized Lists:** Critical for handling 10,000+ songs efficiently.
- **SQLite + SQLModel:** Used for high-performance local metadata storage and retrieval.

---

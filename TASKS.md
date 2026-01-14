# Tremors Music - Tasks

> **Project:** Tremors Music  
> **Version:** 2.0.1  
> **Last Updated:** 2026-01-14

---

### 🔴 Critical Priority

#### Security
- [x] **CORS Wildcard Origins** - `main.py` uses `allow_origins=["*"]` which allows any origin. Should restrict to `http://localhost:5173` and `http://127.0.0.1:8000` in development, and Tauri protocol in production.
- [x] **Disabled CSP** - `tauri.conf.json` has `"csp": null` which disables Content Security Policy entirely. Should configure proper CSP for production builds.
- [x] **Bare Exception Handling** - `scanner.py:180`, `scanner.py:308`, `media.py:69`, `media.py:155` use bare `except:` clauses which can hide errors and mask unexpected exceptions.

#### Bugs
- [ ] **Scanner Stop Not Working** - `handleStopScan()` in `ScannerControl.tsx` only stops polling but never calls the backend `POST /library/scan/stop` endpoint, so the scan continues running in the background.
- [ ] **Lyrics "Searching" Stuck State** - In `FullScreenPlayer.tsx:281`, displays "Searching for lyrics..." indefinitely because the `lyrics === null` check doesn't distinguish between "loading" and "failed to load" states.
- [ ] **Queue Shuffle Desync** - `addToQueue()` in `playerStore.ts` inserts after current song in `queue` but appends to end of `originalQueue`, causing desync when shuffle is toggled off.

### 🟠 High Priority

#### Code Quality
- [ ] **Type Inconsistency** - `getPlaylistSongs(id: string)` in `api.ts:145` uses `string` ID while all other functions use `number`. Should standardize to `number`.
- [ ] **Duplicate Fisher-Yates Shuffle** - Implementation exists in both `playerStore.ts:60-64` (inline) and `utils.ts:68-74` (as `shuffleArray`). Should consolidate.
- [ ] **Duplicate `app_dir` Resolution** - Same logic for determining executable directory exists in `main.py:15-20` and `database.py:6-13`. Should extract to shared utility.
- [ ] **Frontend Version Mismatch** - `package.json` shows version `1.5.0` while all docs and backend show version `2.0.1`. Should sync.
- [ ] **Weak Synced Lyrics Detection** - `scanner.py:235,285` uses simple `'[' in lyrics and ']' in lyrics` which can false-positive on non-LRC content. Should use regex pattern like `\[\d{2}:\d{2}`.

#### Architecture
- [ ] **LibraryPathManager saveEdit Incomplete** - `LibraryPathManager.tsx:74-80` has a TODO comment noting backend doesn't have edit endpoint, but `PATCH /library/paths/{id}` actually exists in `library.py:43-57`. Should connect frontend to existing backend.

### 🟡 Medium Priority

#### UI/UX Polish
- [ ] **Alert() Calls Instead of Toasts** - Native `alert()` used in:
  - `QueuePanel.tsx:143` - "Saved as" confirmation
  - `SettingsModal.tsx:39` - "Invalid path or server error"
  - `LibraryPathManager.tsx:42` - "Invalid path or server error"
  Should replace with toast notifications using existing `useToastStore`.
- [ ] **NaN Guard in Progress Bar** - `Player.tsx:176` calculates `(currentTime / duration) * 100` which produces `NaN` when `duration=0`. Should add guard: `((currentTime / (duration || 1)) * 100)`.
- [ ] **Missing Image Alt Attributes** - Cover images lack meaningful alt text in:
  - `FullScreenPlayer.tsx:145` - Empty or missing
  - `SongList.tsx:210` - Empty `loading="lazy"` only
  - `SearchPage.tsx:244` - Empty `alt=""`
  Should add `alt={song.title}` or similar.

#### Performance
- [ ] **Raw setInterval for Scan Polling** - `ScannerControl.tsx:119` uses `setInterval` for scan status polling. Consider migrating to TanStack Query with `refetchInterval` for better state management and automatic cleanup.
- [ ] **Cover Cache Never Purged** - `media.py:20-21` creates `covers/` cache directory but never cleans up old/orphaned covers when albums are deleted. Should add cleanup logic to library reset or scanner.

#### Code Quality
- [ ] **Unused currentIndex State** - `playerStore.ts:14` defines `currentIndex` but it's calculated dynamically in most places using `queue.findIndex()`. Should either use consistently or remove.
- [ ] **Empty lib.rs** - `tauri/src-tauri/src/lib.rs` likely exists but not reviewed. Should verify sidecar spawn logic is properly implemented.

### � Low Priority

#### Accessibility
- [ ] **Missing Keyboard Navigation** - Full screen player controls (`FullScreenPlayer.tsx`) lack keyboard event handlers beyond global shortcuts.
- [ ] **No ARIA Labels** - Interactive elements like icon buttons (`IconButton.tsx`, various components) lack `aria-label` attributes for screen readers.
- [ ] **Color Contrast** - Some `text-apple-subtext` on dark backgrounds may not meet WCAG AA contrast requirements.

#### Code Quality
- [ ] **Console.error Without User Feedback** - `LibraryPathManager.tsx:26,59` logs errors but shows nothing to users. Should add toast notification.
- [ ] **Hardcoded Magic Numbers** - Various places use hardcoded limits:
  - `library.py:135` - Recently added limit 50
  - `library.py:243` - Most played limit 50
  - `QueuePanel.tsx:256` - Queue display limit 50
  Should consider extracting to constants.

#### Documentation
- [ ] **README Version Badges** - Badges show `React-19.2.0`, `FastAPI-0.115.0`, `Tauri-2.0.1` but should verify these match actual dependency versions.
- [ ] **CHANGELOG Year Inconsistency** - Dates show `2025-12-23` but "Last Updated" shows `2026-01-13`. Verify timeline accuracy.
- [ ] **API Documentation Missing** - `DEVELOPMENT.md` lists endpoints but doesn't document request/response schemas.

---

## 🐛 Bug Fixes (Previously Tracked)

- [ ] **Scanner Stop Never Called** - `handleStopScan()` in `ScannerControl.tsx` needs to call backend endpoint (duplicate of above, can merge).

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

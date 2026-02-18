# Tremors Music - Tasks

> **Project:** Tremors Music  
> **Version:** 2.0.2
> **Last Updated:** 2026-02-18

---

## 🔴 Critical / Security

#### Path Traversal & Injection
- [ ] **Path Traversal in Library Path Addition** - `library.py` `add_path()` only checks `os.path.isdir()` but doesn't sanitize against traversal like `..` or symlink attacks. A malicious path could expose sensitive directories.
- [ ] **Path Traversal in Cover Serving** - `media.py` `get_cover()` constructs file paths using user-controlled `album_id` without validating the resolved path stays within the covers directory. Could serve arbitrary files.
- [ ] **SQL Injection via `ilike` Patterns** - `library.py` `search()` passes `q` directly into `.ilike(f"%{q}%")` — special SQL LIKE characters (`%`, `_`) are not escaped, allowing pattern manipulation.
- [ ] **Unrestricted Library Path Deletion** - `library.py` `remove_path()` deletes a library path and all associated songs/albums without verifying the path belongs to the requesting user context. In multi-user scenarios this is dangerous.

#### Unvalidated Input
- [ ] **No Input Validation on Playlist Names** - `playlists.py` accepts any string for playlist names including empty strings, extremely long strings, or strings with control characters.
- [ ] **No Rate Limiting on API Endpoints** - All endpoints (scan, reset, delete) have no rate limiting. A malfunctioning frontend or rogue client could spam destructive operations.
- [ ] **Hard Reset Deletes Entire DB Without Authentication** - `library.py` `reset_library(hard=True)` wipes all songs, albums, and playlists with a single unauthenticated HTTP DELETE.

#### CSP Weaknesses
- [ ] **Unsafe CSP Directives** - `tauri.conf.json` CSP includes `'unsafe-inline'` and `'unsafe-eval'` for scripts, which significantly weakens XSS protection.

---

## 🟠 High Priority

#### Bugs
- [ ] **Scanner Race Condition** - `scanner.py` uses a module-level `_stop_scan_flag` without proper thread synchronization. If two scan requests arrive simultaneously, both could start because the `is_scanning` check and set are not atomic.
- [ ] **Scanner Thread Never Joined** - `scanner.py` `_scanner_thread` reference is stored but never `.join()`ed. On app shutdown during a scan, the thread is abandoned, potentially corrupting the database.
- [ ] **Album ID Extraction** - `media.py:50` uses `filename.split('_')[0]` which fails for direct IDs (e.g. `123.jpg`). Should use `os.path.splitext` first.
- [ ] **Stale Synced Lyrics** - `scanner.py` updates synced lyrics only if present, but doesn't clear them if removed from audio tags.
- [ ] **Invalid Cover Request** - `FullScreenPlayer.tsx` requests `/covers/0` when `album_id` is null. Should guard with a check.
- [ ] **`clearQueue` Stops Playback Abruptly** - `playerStore.ts` `clearQueue()` wipes the queue and current song without stopping the `<audio>` element, potentially causing errors if audio continues referencing a cleared song.
- [ ] **Playlist Reorder Allows Invalid Song IDs** - `playlists.py` `reorder_songs()` accepts arbitrary `song_ids` without verifying they belong to the playlist, allowing injection of unrelated songs.
- [ ] **`removeFromQueue` Index Mismatch During Shuffle** - `playerStore.ts` `removeFromQueue()` splices from the active queue but doesn't update `_originalQueue`, causing desync when shuffle is toggled off.
- [ ] **Excessive Invalidation** - `ScannerControl.tsx` invalidates all library queries on mount because `!isScanning` is true initially. Should only invalidate after a scan completes.
- [ ] **Delete Cascade Missing for Albums** - `library.py` `reset_library()` with `hard=True` deletes songs and albums but doesn't delete `PlaylistSong` entries, leaving orphaned junction table rows.

#### Sidecar / Tauri
- [ ] **Sidecar Panic Safety** - `lib.rs` uses `expect()` for sidecar spawning. If the backend binary is missing or fails, the entire app crashes without user feedback.
- [ ] **Sidecar Cleanup** - `lib.rs` `on_window_event(Destroyed)` has an empty `#[cfg(not(debug_assertions))]` block that doesn't actually kill the backend process. Orphaned sidecar processes persist after app close.
- [ ] **No Sidecar Health Check** - Frontend connects to `localhost:8000` with no retry/backoff. If the sidecar is slow to start, the UI shows errors.

---

## 🟡 Medium Priority

#### Performance
- [ ] **Full Table Scan on Search** - `library.py` `search()` uses `ilike` on `title`, `artist`, `album` fields without database indexes on these columns, causing full table scans on large libraries.
- [ ] **Unbounded Song Listing** - `library.py` `get_songs()` returns all songs regardless of library size. For 10k+ songs this sends a massive JSON payload. Should implement server-side pagination.
- [ ] **N+1 Query in Album Listing** - `library.py` `get_albums()` runs a separate `func.count()` subquery for each album. Should use a single JOIN/GROUP BY query.
- [ ] **Cover Image Processing on Every Request** - `media.py` `get_cover()` reads, resizes, and converts cover images on every HTTP request with no caching headers. Should return `Cache-Control` or `ETag` headers.
- [ ] **Unresized Embedded Covers** - `media.py` `get_cover()` extracts from audio files and resizes to 600x600, but this CPU-intensive operation happens per-request with no caching of the result.
- [ ] **`useGlobalKeyHandler` Re-registers on Every State Change** - `useGlobalKeyHandler.ts` depends on many store values, causing frequent `addEventListener`/`removeEventListener` cycles.
- [ ] **Queue Panel Renders All Items** - `QueuePanel.tsx` renders up to 50 items without virtualization (unlike `SongList.tsx` which uses `react-virtuoso`). Large queues cause DOM bloat.

#### Code Quality
- [ ] **Console.error Without User Feedback** - `LibraryPathManager.tsx:26,59` logs errors but shows nothing to users. Should add toast notification.
- [ ] **Inconsistent Alerts** - `LibraryPathManager.tsx:42,82` uses native `alert()` instead of the Toast system used everywhere else.
- [ ] **Hardcoded Magic Numbers** - Various places use hardcoded limits:
  - `library.py:135` - Recently added limit 50
  - `library.py:243` - Most played limit 50
  - `QueuePanel.tsx:256` - Queue display limit 50
  Should be extracted to named constants.
- [ ] **Unused `aiofiles` Dependency** - `pyproject.toml` lists `aiofiles>=25.1.0` but it's never imported in any backend file.
- [ ] **Unused `requests` Dependency** - `pyproject.toml` lists `requests>=2.32.5` but it's never imported. Adds unnecessary attack surface.
- [ ] **Unused `watchdog` Dependency** - `pyproject.toml` lists `watchdog>=6.0.0` but no file watcher is implemented.
- [ ] **Unused `python-multipart` Dependency** - `pyproject.toml` lists `python-multipart` but no file upload endpoints exist.
- [ ] **Dead `saveEdit` Function Suppressed** - `LibraryPathManager.tsx:73` has `@typescript-eslint/no-unused-vars` suppression on `saveEdit` but the function IS used in the template. The suppress is likely stale.
- [ ] **Module-level Side Effects in `database.py`** - `database.py` creates the engine and database file at import time (module level), making testing impossible and causing issues if imported before the app directory is ready.
- [ ] **Bare String Error Handling** - `scanner.py` catches exceptions broadly in `_process_single_file()` and converts to string, losing stack trace information useful for debugging.
- [ ] **`ScanProgress` Mixes Concerns** - `scanner_progress.py` uses both `threading.Lock` and `dataclass` mutation, but `get_status()` copies fields without holding the lock for the entire read, allowing torn reads of progress state.
- [ ] **`SortableSongItem` Calls `usePlayerStore` Per Item** - `QueuePanel.tsx:39` each sortable item calls `usePlayerStore()` individually instead of receiving the function via props, causing unnecessary store subscriptions.

#### Data Integrity
- [ ] **No Foreign Key Enforcement** - SQLite doesn't enforce foreign keys by default. `database.py` never executes `PRAGMA foreign_keys = ON`, so `album_id` references in songs can become dangling.
- [ ] **No Unique Constraint on Song Path** - `models.py` `Song` model doesn't enforce uniqueness on `path` field. If the scanner runs concurrently or has a bug, duplicate songs can be inserted.
- [ ] **Playlist Song Order Gaps** - `playlists.py` `add_songs()` calculates max order once, then increments for each song. If two concurrent `add_songs` calls arrive, they could assign overlapping order values.
- [ ] **Album Orphan Cleanup Incomplete** - `scanner.py` `_cleanup_orphaned_albums()` deletes albums with no songs, but `_cleanup_missing_files()` doesn't remove the album's cover file from disk.

---

## ⚫ Low Priority

#### Accessibility
- [ ] **Missing Keyboard Navigation** - Full screen player controls (`FullScreenPlayer.tsx`) lack keyboard event handlers beyond global shortcuts.
- [ ] **No ARIA Labels** - Interactive elements like icon buttons (`IconButton.tsx`, various components) lack `aria-label` attributes for screen readers.
- [ ] **Color Contrast** - Some `text-apple-subtext` on dark backgrounds may not meet WCAG AA contrast requirements.

#### Documentation
- [ ] **README FastAPI Version Badge Wrong** - Badge shows `FastAPI-0.115.0` but `pyproject.toml` has `fastapi>=0.121.3`. Tauri badge shows `2.0.2` (correct) but was `2.0.1` previously.
- [ ] **CHANGELOG Version Mismatch** - `CHANGELOG.md` header says `Version: 2.0.1` (line 4) but current version is `2.0.2`. The `[2.0.2]` section exists in the body.
- [ ] **CHANGELOG Year Inconsistency** - Dates show `2025-12-23` but "Last Updated" shows `2026-01-14`. Verify timeline accuracy.
- [ ] **API Documentation Missing Schemas** - `DEVELOPMENT.md` lists endpoints but doesn't document request/response schemas.
- [ ] **pyproject.toml Placeholder Description** - Backend `pyproject.toml` still has `"Add your description here"` as description.
- [ ] **DEVELOPMENT.md Tauri Dev Note Incorrect** - States "This command automatically starts the Frontend and Backend in the background" but `tauri.conf.json` `beforeDevCommand` only starts the frontend, not the backend.
- [ ] **DEVELOPMENT.md Missing Scanner Progress/Media Endpoints** - API routes section lists `/library/scan/status` and `/library/scan/stop` but omits detailed docs for `/covers/cache/cleanup` endpoint.
- [ ] **Missing `__init__.py` in Router Package** - `backend/router/` has no `__init__.py`. Works because of how FastAPI imports, but is a Python packaging anti-pattern that could break with different import methods.
- [ ] **PRIVACY.md References Non-Existent Features** - Should verify all claims match current implementation.

#### Frontend Code Quality
- [ ] **`useEffect` Eslint Suppressions** - Multiple components (`LibraryPathManager.tsx`, `ScannerControl.tsx`, `SyncedLyrics.tsx`) suppress `react-hooks/set-state-in-effect` rather than refactoring to avoid the pattern.
- [ ] **Inconsistent Error Handling in API Calls** - Some components use `try/catch` with toast, others with `console.error`, others with `alert()`. No unified error handling strategy.
- [ ] **Genre Page Fetches All Songs Then Filters Client-Side** - `GenrePage.tsx` likely loads all songs for a genre without server-side filtering, which is inefficient for large genres.
- [ ] **Context Menu Positioning Can Overflow Viewport** - `ContextMenu.tsx` positions at click coordinates without checking if the menu extends beyond the window boundary.

#### Backend Code Quality
- [ ] **`get_session` Generator Doesn't Handle Exceptions** - `database.py` `get_session()` yields the session but doesn't wrap in try/finally, so if an endpoint handler raises an exception, the session may not be properly closed.
- [ ] **`scan_directory` Commits Per-File** - `scanner.py` calls `session.commit()` after each individual file instead of batching. For large libraries (10k+ files) this means 10k+ individual disk writes.
- [ ] **`_extract_cover` Silent Failure** - `scanner.py` `_extract_cover()` catches all exceptions and returns `None` silently. Cover extraction failures produce no user-visible feedback.
- [ ] **Logging Format String Style** - `main.py` uses `%`-style format strings for logging (correct), but `scanner.py` mixes f-strings with logging calls, which evaluates the f-string even if the log level is disabled.

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

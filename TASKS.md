# Future Tasks (CodeRabbit Review)

Extracted from PR review comments. Grouped by priority.

---

## 🔴 Critical / Major

- [ ] **ScannerControl.tsx**: `handleStopScan` doesn't call backend `/scan/stop` endpoint - only stops polling
- [ ] **api.ts**: `getPlaylistSongs(id: string)` inconsistent with other playlist functions using `number`
- [ ] **AlbumDetail.tsx**: Add validation to prevent non-numeric IDs (`enabled: !!id && !isNaN(Number(id))`)
- [ ] **types.ts vs api.ts**: Consolidate duplicate `Artist` interface definitions

---

## 🟠 Major

- [ ] **FullScreenPlayer.tsx**: "Searching for lyrics..." shown indefinitely if fetch fails - add loading/error states
- [ ] **Player.tsx**: Guard against NaN in progress bar (`duration > 0 ? ... : 0`)
- [ ] **LibraryPathManager.tsx**: `saveEdit` is a no-op - hide Edit button until backend ready
- [ ] **SettingsModal.tsx**: Remove incorrect ESLint rule name `react-hooks/set-state-in-effect`
- [ ] **playerStore.ts**: `addToQueue` inserts at different positions in `queue` vs `originalQueue`

---

## 🟡 Minor / Nitpick

### Error Handling
- [ ] **SettingsModal.tsx:43-46**: Add try/catch for `handleRemovePath`
- [ ] **AllPlaylistsPage.tsx:62-78**: Add try/catch for `deletePlaylist`
- [ ] **SmartPlaylistDetail.tsx:75-79**: Add error handling for `toggleFavorite`
- [ ] **SongList.tsx:74-78**: Add error handling for `toggleFavorite`
- [ ] **PlaylistDetail.tsx:232-246**: Add error handling for `deletePlaylist`

### Replace `alert()` with Toast
- [ ] **LibraryPathManager.tsx:35-44**: Replace `alert()` with `addToast()`
- [ ] **QueuePanel.tsx:135-149**: Replace `alert()` with toast

### Accessibility (alt attributes)
- [ ] **GenreDetail.tsx:134-140**: Add `alt` attribute to cover image
- [ ] **SmartPlaylistDetail.tsx:181-183**: Add `alt` attribute to cover image

### Code Quality
- [ ] **main.py:12-50**: Extract `app_dir` resolution to `get_app_dir()` utility
- [ ] **IconButton.tsx**: Add `type="button"` default to prevent form submits
- [ ] **database.py:15-18**: Rename `sqlite_file_name` to `sqlite_file_path`
- [ ] **migrate_db.py:28**: Catch `sqlite3.Error` instead of bare `Exception`
- [ ] **playerStore.ts:56-77**: Use `shuffleArray` utility instead of inline shuffle
- [ ] **ArtistsPage.tsx:52-68**: Import `shuffleArray` statically instead of dynamic import
- [ ] **ThemeSettings.tsx:9-17**: Extract duplicate color palette to shared constant
- [ ] **App.tsx:29-34**: Use `data-search-input` attribute instead of placeholder query

### UX Improvements
- [ ] **SettingsModal.tsx:48-52**: Fixed 2s timeout doesn't reflect actual scan completion
- [ ] **ContextMenu.tsx:74-80**: Toast message may show incorrect state (use API response)
- [ ] **ContextMenu.tsx:26-34**: Menu position calculation may cause layout shift
- [ ] **ConfirmDialog.tsx:10-23**: Enter key confirmation may trigger unintended behavior
- [ ] **confirmStore.ts:25-33**: Handle concurrent dialog requests (resolve pending promise)
- [ ] **ScannerControl.tsx:157**: Remove `getElapsedTime()` indirection
- [ ] **ScannerControl.tsx:237-259**: Separate `showErrors` state for live vs last-scan

### Fragile Patterns
- [ ] **SyncedLyrics.tsx:142-148**: Global `document.querySelector('audio')` is fragile
- [ ] **SongList.tsx:163**: Hardcoded `calc(100vh - 280px)` may break with layout changes
- [ ] **useKeyboardShortcuts.ts:51-52**: `navigator.platform` is deprecated
- [ ] **useKeyboardShortcuts.ts:127-132**: Mute toggle loses previous volume level
- [ ] **useKeyboardShortcuts.ts:161-169**: Options object in deps may cause re-renders

### Backend
- [ ] **playlists.py:82-98**: Add playlist existence check before removing songs
- [ ] **media.py:45-70**: Replace bare `except` with specific exceptions + logging
- [ ] **media.py:139-156**: Replace bare `except` with specific exception handling
- [ ] **media.py:161-162**: Synced lyrics detection may have false positives (use regex)
- [ ] **scanner.py:50**: Split statement onto separate lines (E701)
- [ ] **scanner.py:180-181**: Bare except clause - catch specific exceptions
- [ ] **scanner.py:234-236**: Improve synced lyrics detection heuristic
- [ ] **scanner.py:193-196**: Add explicit `str()` conversion for defensive robustness
- [ ] **scanner_progress.py:48-55**: Log warning when error limit reached
- [ ] **library.py:229-238**: Use `date_added` instead of `id` for recently added

### Types & Consistency
- [ ] **QueuePanel.tsx:71-75**: Handle missing `album_id` instead of fallback to 0
- [ ] **QueuePanel.tsx:223-228**: Same album ID fallback issue
- [ ] **api.ts:53-58**: Remove duplicate `Artist`/`Genre` interfaces (use types.ts)
- [ ] **api.ts:4-12**: Production URL hardcoded to 127.0.0.1
- [ ] **models.py:34-57**: `SongListItem.format` optional vs `Song.format` required

### Documentation
- [ ] **README.md**: Change "Full Screen" to "Full-screen" when used as adjective

---

## ✅ Already Fixed (This PR)
- [x] ESLint errors (14 issues across 8 files)
- [x] CodeRabbit critical issues (12 issues)
- [x] TypeScript build passing

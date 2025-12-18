# Tasks & Known Issues

Tracking known issues and planned improvements for Tremors Music v2.0.0.

---

## ✅ Completed (v2.0.0)

- [x] Consolidated web and desktop repos into monorepo
- [x] Fresh Tauri initialization with proper branding
- [x] Build system working (npm run build)
- [x] Memory optimizations from web version
- [x] Custom icons and installer branding
- [x] Updated all documentation

---

## 🔴 High Priority

### Bugs

- [ ] **ScannerControl.tsx**: `handleStopScan()` (line 148) doesn't call backend `/scan/stop` endpoint - only stops polling. The backend endpoint exists at `/library/scan/stop` but is never called.
- [ ] **api.ts**: `getPlaylistSongs(id: string)` uses `string` type (line 145) while all other ID parameters use `number` - type inconsistency that could cause issues.
- [ ] **FullScreenPlayer.tsx**: "Searching for lyrics..." shown indefinitely if fetch fails (line 272). The `lyrics` state is only set to `null` on song change, not on fetch failure.

### Logic Issues

- [ ] **playerStore.ts**: `addToQueue()` (lines 142-158) inserts after current song in `queue` but appends to end of `originalQueue` - causes desync when shuffle is toggled.

---

## 🟠 Medium Priority

### Incomplete Features

- [ ] **LibraryPathManager.tsx**: `saveEdit()` (line 74) is a no-op with TODO comment, but backend endpoint `PATCH /library/paths/{id}` already exists (library.py:43-57). Just needs to call `api.patch()`.

### UI/UX Issues

- [ ] **Player.tsx**: Add guard against NaN in progress bar calculation (`duration > 0 ? ... : 0`) - currently could show NaN% width.
- [ ] Replace 3 `alert()` calls with toast notifications:
  - `QueuePanel.tsx:143` - "Saved as..." message
  - `SettingsModal.tsx:39` - "Invalid path or server error"
  - `LibraryPathManager.tsx:42` - "Invalid path or server error"

---

## 🟡 Low Priority / Nice to Have

### Accessibility

- [ ] Add meaningful `alt` attributes to cover images. Currently:
  - `FullScreenPlayer.tsx:136` - no alt
  - `SongList.tsx:210` - no alt
  - `SearchPage.tsx:244` - empty alt=""

### Code Quality

- [ ] **scanner.py**: Use regex for synced lyrics detection instead of simple `[` and `]` heuristic (lines 235, 285). Current check could false-positive on songs with brackets in lyrics.
- [ ] Extract `app_dir` resolution to shared utility (duplicated in `main.py` and potentially elsewhere).
- [ ] Remove unused variable `_id` in `saveEdit()` function (LibraryPathManager.tsx:74).

### Minor Improvements

- [ ] Deduplicate Fisher-Yates shuffle implementation (exists in both `playerStore.ts` and `utils.ts`).
- [ ] Consider adding loading skeleton for lyrics panel in FullScreenPlayer.

---

## 📝 Future Features

- [ ] File watcher for auto-updating library on file changes
- [ ] Equalizer controls (Web Audio API)
- [ ] Mini player mode
- [ ] macOS and Linux builds
- [ ] Keyboard shortcuts help modal (? key)
- [ ] Export/import playlists

---

## 🔧 Technical Debt

- [ ] Consider using TanStack Query for scan status polling instead of raw `setInterval`.
- [ ] Consolidate type definitions - some types defined in both `api.ts` and `types.ts` (e.g., Artist, Genre).
- [ ] Add error boundaries around individual components, not just at App level.

---

## 📊 Performance Notes

- Virtualized list in SongList handles 10,000+ songs efficiently
- Album covers use `loading="lazy"` for deferred loading
- SongListItem model excludes lyrics to reduce payload size
- Scanner commits every 50 songs for transaction optimization

---

*Last reviewed: 2024-12-18*

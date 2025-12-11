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

- [ ] **ScannerControl.tsx**: `handleStopScan` doesn't call backend `/scan/stop` - only stops polling
- [ ] **api.ts**: `getPlaylistSongs(id: string)` inconsistent with other functions using `number`
- [ ] **FullScreenPlayer.tsx**: "Searching for lyrics..." shown indefinitely if fetch fails

---

## 🟠 Medium Priority

- [ ] **Player.tsx**: Guard against NaN in progress bar (`duration > 0 ? ... : 0`)
- [ ] **playerStore.ts**: `addToQueue` inserts at different positions in `queue` vs `originalQueue`
- [ ] **LibraryPathManager.tsx**: `saveEdit` is a no-op - hide Edit button until backend ready

---

## 🟡 Low Priority / Nice to Have

- [ ] Replace `alert()` calls with toast notifications
- [ ] Add `alt` attributes to all cover images for accessibility
- [ ] Extract `app_dir` resolution to shared utility
- [ ] Use regex for synced lyrics detection instead of simple heuristic

---

## 📝 Future Features

- [ ] File watcher for auto-updating library on file changes
- [ ] Equalizer controls (Web Audio API)
- [ ] Mini player mode
- [ ] macOS and Linux builds

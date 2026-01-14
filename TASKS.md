# Tremors Music - Tasks

> **Project:** Tremors Music  
> **Version:** 2.0.2
> **Last Updated:** 2026-01-14

---

### ⚫ Low Priority

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

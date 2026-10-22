# Tremors Music Task Backlog

> **Version:** 0.1.0 | **Last Updated:** 2026-10-20

10 open findings and four release checks. Only unfinished work is listed. Task IDs are retained for reference; recheck earlier evidence before implementation.

Code paths are relative to `tremors-music-app/`. See [DEVELOPMENT.md](DEVELOPMENT.md#verification) for test commands, manual checklists, and recorded results.

## Release verification

- [ ] Check Windows rendering, display scaling, physical audio, SMTC, and installer install/upgrade/uninstall behavior.
- [ ] Check Linux physical audio, hardware rendering, Wayland, and native folder-picker portals.
- [ ] Measure startup, scanning, search/sort, scrolling, CPU, and peak memory with a large real library. Profile `browsing::build_rows`, full library snapshots, and `Queue::reconcile` before optimizing.
- [ ] After verification, produce and publish matching binaries, corresponding source, and checksums.

## Library and persistence

- [ ] **CORE-001 | High | Prevent song ID reuse.**
  - **Location:** `crates/core/src/library.rs` (`Library::open`, `apply_scan`); `crates/core/src/queue.rs`.
  - **Current:** Recyclable SQLite IDs identify saved queue entries. Previously reproduced: a new track inherited a deleted track's saved queue position.
  - **Change:** Use identities that cannot be recycled and handle existing sessions during migration.
  - **Verify:** Highest-ID deletion/reimport, multiple roots, folder removal, and missing queued songs.

- [ ] **CORE-002 | Medium | Restore watches after folder replacement.**
  - **Location:** `crates/core/src/service.rs` (`worker`).
  - **Current:** Watch registrations survive root disappearance or replacement. Automatic imports previously stopped after recreating a watched directory on Windows.
  - **Change:** Invalidate stale registrations, retry with bounded backoff, and surface watcher setup failures.
  - **Verify:** Folder recreation, drive reconnects, initially unavailable roots, and watching toggles.

- [ ] **CORE-003 | Medium | Recover corrupt artwork caches.**
  - **Location:** `crates/core/src/scanner.rs` (`scan_cached`, `cache_cover`).
  - **Current:** Existing cache paths are trusted and PNGs are written directly to their final names. A truncated cached image was previously retained on rescan.
  - **Change:** Publish cache files atomically and regenerate invalid images.
  - **Verify:** Truncated files, interrupted writes, shared artwork, and unchanged-track reuse.

- [ ] **CORE-004 | Medium | Preserve filesystem paths losslessly.**
  - **Location:** `crates/core/src/library.rs`.
  - **Current:** Paths are stored with `to_string_lossy()`. Distinct Windows paths containing unpaired UTF-16 units previously collapsed into one record; Linux non-UTF-8 paths still need testing.
  - **Change:** Store paths losslessly or reject unsupported paths before changing records.
  - **Verify:** Round trips, uniqueness, scans, pruning, relocation, and existing-data migration.

- [ ] **CORE-005 | Medium | Order shutdown saves with worker commands.**
  - **Location:** `crates/desktop/src/ui/runtime.rs` (`MusicApp::drop`); `crates/core/src/service.rs`.
  - **Current:** Shutdown saves through a second connection while the worker can still process older saves. This ordering risk is established from source; a lost-save reproduction remains outstanding.
  - **Change:** Add an acknowledged shutdown/flush, save session and preferences together, and report failures through diagnostics.
  - **Verify:** Immediate exit after changes, exit during scanning, pending saves, and database contention.

## Desktop workflows

- [ ] **UI-001 | High | Make primary workflows keyboard-accessible.**
  - **Location:** `crates/desktop/src/ui/views/`.
  - **Current:** Windows inspection of custom navigation, rows, controls, and overlays previously found missing accessible names/states and incomplete keyboard and modal-focus handling.
  - **Change:** Add roles, labels, states, keyboard activation, visible focus, and modal focus restoration.
  - **Verify:** All navigation, playlist actions, player controls, lyric seeking, and dialogs with keyboard-only input and a screen reader.

- [ ] **UI-002 | Medium | Refresh media metadata after tag edits.**
  - **Location:** `crates/desktop/src/ui/runtime.rs` (`MusicApp::poll`).
  - **Current:** The desktop now compares a revision of published metadata fields, including title, artist, album, cover, and duration, and refreshes after editor saves. Native same-track rescan observations remain outstanding.
  - **Change:** Implemented in `crates/desktop/src/media.rs`; verify on each native media-control backend.
  - **Verify:** Same-ID rescans while playing and paused on Windows and Linux without repeatedly publishing unchanged state.

- [ ] **UI-003 | Medium | Show playback errors in fullscreen.**
  - **Location:** `crates/desktop/src/ui/views/mod.rs`; `crates/desktop/src/ui/views/fullscreen.rs`.
  - **Current:** The status area is hidden in immersive mode with no replacement error feedback. Failure-path runtime checks remain outstanding.
  - **Change:** Show failures with a useful retry or return action.
  - **Verify:** Missing/unreadable tracks, decoding failures, device errors, and stopped workers in both layouts.

## Windows backend

- [ ] **WIN-001 | Medium | Fix refresh-rate fallback division by zero.**
  - **Location:** `third-party/gpui-pre-windows/src/vsync.rs` (`retrieve_duration`).
  - **Current:** Frequency is divided by 1,000,000 before conversion. Rational rates such as 60/1 produce a zero divisor; the helper previously reproduced the panic. A physical DWM fallback trigger remains unverified.
  - **Change:** Use checked rational duration conversion.
  - **Verify:** Common and fractional refresh rates, QPC values, invalid frequencies, overflow, and the DWM fallback path.

- [ ] **WIN-002 | Medium | Investigate clipboard ownership.**
  - **Location:** `third-party/gpui-pre-windows/src/clipboard.rs`.
  - **Current:** Clipboard writes use `OpenClipboard(None)` before `EmptyClipboard` and `SetClipboardData`. Native copy/cut round trips remain unverified.
  - **Change:** Reproduce with search and playlist inputs. If confirmed, use a valid owner window and preserve memory ownership.
  - **Verify:** Copy/cut/paste, clipboard contention, and error reporting.

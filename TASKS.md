# Tremors Music - Tasks

Track unfinished implementation, platform checks, and current verification results here. See [DEVELOPMENT.md](DEVELOPMENT.md#verification) for checks and acceptance criteria.

## Release verification

- [ ] Verify native Windows rendering, scaling, physical audio, SMTC, and installer lifecycle using the [manual checklists](DEVELOPMENT.md#desktop-manual-checklist).
- [ ] Verify Linux physical audio, hardware rendering, Wayland, and desktop portal behavior.
- [ ] Measure responsiveness on a large real library. Profile scans, search/sort, full snapshots after favorites/play-count changes, and cloned song/lyric data in `MusicApp::rebuild` and `Queue::reconcile`; record timings and peak memory before choosing optimizations. Accessibility and keyboard-focus work is tracked in UI-001 below.
- [ ] Publish matching binaries, corresponding source, and checksums after verification.

## Architecture follow-up

- [ ] Split desktop `ui.rs` into focused library, player, playlist, settings, and shared-view modules while preserving behavior and keeping composition in the desktop shell. The reviewed file has 2,716 physical lines spanning worker events, persistence, navigation, keyboard actions, overlays, and rendering. Establish behavior checks for these boundaries before extraction; keep this follow-up separate from defect fixes.
- [ ] Review core module boundaries as features grow; keep GPUI and platform presentation outside `tremors-core`.

## Future enhancements

- [ ] Decide whether to add WMA and Opus.
- [ ] Consider gapless playback, output-device selection, an equalizer, tray support, and custom global shortcuts.
- [ ] Consider AppImage distribution, signing, and automatic updates.

## Repository review findings

Reviewed on 2026-10-10 at `de79c0a`, starting from a clean checkout. All 104 tracked files are accounted for in the coverage ledger below. No application fixes, version changes, release changes, or CI/CD changes were made during the initial review. The subsequent branding fix is marked complete below; other findings remain open.

Priority: **High** affects library identity or access to primary workflows; **Medium** affects a specific supported workflow or failure condition. Evidence is identified as a reproduction, native observation, source finding, or investigation so that untested behavior is not presented as a runtime result.

Rust and build-script locations in findings are relative to `tremors-music-app/`; other paths are relative to the repository root.

### Library and persistence

- [ ] **CORE-001 | High | Prevent deleted song IDs from being reused by another track.** Evidence: `crates/core/src/library.rs:52,185,240` declares `songs.id INTEGER PRIMARY KEY`, deletes records during pruning, and persists queue IDs independently in JSON. `Queue::restore` and `Queue::reconcile` resolve those IDs without checking track identity. A temporary harness using the production APIs saved track ID 1 at 20 seconds, pruned it, inserted a different track, and restored the different track as ID 1 at 20 seconds. Reconciliation can also replace queue metadata while the old audio is loaded. Use identities that cannot be recycled and handle existing saved sessions during migration. Verify highest-ID deletion/reimport, sequential scans of multiple roots, folder removal, and restoration with missing tracks. [SQLite's identity-allocation documentation](https://sqlite.org/autoinc.html) supports the observed behavior.

- [ ] **CORE-002 | Medium | Recover folder watches after a directory is replaced.** Evidence: `crates/core/src/service.rs:99-132` retains a path in `watched` until the configured root is removed; disappearance and watcher errors do not invalidate it. A Windows harness first confirmed ordinary updates from one to two tracks, renamed the watched directory, recreated the same path, and added a third track. After two eight-second observation windows, the filesystem contained three tracks but a refresh still returned two. Invalidate stale registrations, retry with bounded backoff, and surface watcher creation/registration failures currently discarded by `.ok()` and `.is_ok()`. Verify recreation, removable-drive reconnect, initially unavailable roots, and disabling/re-enabling watching.

- [ ] **CORE-003 | Medium | Regenerate corrupt artwork and publish cache files atomically.** Evidence: `crates/core/src/scanner.rs:98-101,239-256` accepts an existing cover solely because its path exists and writes a new PNG directly to its final name. A harness imported `tone-art.flac`, truncated its temporary cached PNG, and confirmed that both `scan_cached` and a direct `read_song` retained the zero-byte file. An interrupted write can therefore leave permanently broken artwork. Write through a temporary file, validate failed/existing cache entries before reuse, and regenerate unusable entries. Verify truncated PNGs, interrupted/failed writes, shared artwork, and unchanged-track cache reuse.

- [ ] **CORE-004 | Medium | Preserve filesystem paths without lossy Unicode conversion.** Evidence: `crates/core/src/library.rs:159,174,204-205,270-287,376` stores paths through `to_string_lossy()`. A Windows API-level harness supplied two distinct paths containing different unpaired UTF-16 units; they collapsed into one database record and the returned path matched neither original. Linux filenames with non-UTF-8 bytes have the same representation problem, but that Linux case was not executed here. Store a lossless platform representation, or reject unsupported paths with a clear warning before altering records. Verify path round trips, uniqueness, scanning/pruning, relocation, and migration of existing ordinary paths.

- [ ] **CORE-005 | Medium | Make the final state save ordered with the library worker.** Source finding: `crates/desktop/src/ui.rs:2700-2714` sets cancellation and writes session/preferences through a second database connection while `crates/core/src/service.rs:68-85,161-184` owns a detached worker that can still process earlier queued saves. There is no shutdown acknowledgement or join establishing which save wins; final writes can also time out behind a transaction. Release-mode failures go only to `eprintln!`. Add an ordered shutdown/flush acknowledgement and persist the final session/preferences together, reporting failures through the existing log/error path. Verify immediate exit after a queued change, exit during a scan, pending older saves, and database contention. This ordering risk was established from source; a lost-save runtime reproduction remains outstanding.

### Desktop workflows

- [ ] **UI-001 | High | Expose primary workflows to keyboard and assistive technology.** Evidence: native Windows UI Automation observations of the installed v0.1.0 executable showed no sidebar navigation or song-title nodes, unnamed playback/song-action buttons, and unnamed seek/volume sliders. Settings switches were exposed as buttons without their checked state. The startup log also reported focused elements with missing IDs/roles. `crates/desktop/src/ui.rs:922-1135,1388-1464,1550-1743,1840-1986,1989-2200` uses mouse-only custom rows/cards/navigation and visual overlays without modal focus handling. Add roles, meaningful names and states, keyboard activation, visible focus, and modal focus entry/containment/restoration. Verify every navigation page, song/playlist action, player control, lyric seeking, and confirmation dialog with keyboard-only use and a screen reader. This replaces the accessibility portion of the original release-verification task.

- [ ] **UI-002 | Medium | Refresh desktop media metadata when the current track's tags change.** Source finding: `crates/desktop/src/ui.rs:492-498` publishes metadata only when the song ID changes, while `Queue::reconcile` updates tags/artwork for the same ID after a scan. Editing the current track's title, artist, album, artwork, or duration can therefore leave SMTC/MPRIS metadata stale. Compare the published metadata fields or a revision and republish after relevant updates without spamming unchanged state. Verify a same-ID metadata rescan while playing and paused on both media-control backends. Native SMTC/MPRIS observations for this scenario remain outstanding.

- [ ] **UI-003 | Medium | Show playback failures in the fullscreen player.** Source finding: `crates/desktop/src/ui.rs:305-311,375,426,2553-2609` writes worker/playback errors into `status`, but the status view is rendered only when `immersive` is false. A missing/unreadable track, decoder error, or stopped worker has no visible explanation in fullscreen playback. Provide error feedback and an appropriate retry/return action in that view. Verify unreadable/missing audio, decoding failure, and disconnected worker paths in ordinary and fullscreen presentation. The error-path runtime cases remain outstanding; successful fullscreen rendering was observed.

### Packaging and Windows backend

- [x] **BUILD-001 | Medium | Align Git's tracked branding filenames with their references.** Original evidence: Git tracked `assets/tremorsmusic.svg` and `assets/tremorsmusic.png`, while `README.md:2` requested `assets/TremorsMusic.svg` and `scripts/build-linux.sh:31` copied `assets/TremorsMusic.png`. Windows hid the mismatch, but the exact referenced names were absent from a fresh Git archive. Fixed on 2026-10-10 with explicit case-only Git renames to the documented PascalCase names. Both asset blob IDs are unchanged. Verified that a fresh Git archive contains the exact README and Linux icon paths, has no old lowercase entries, and contains a parseable SVG. Existing website copies already match. Live GitHub rendering awaits publishing the rename; a Linux release build was not rerun for this filename-only change.

- [ ] **WIN-001 | Medium | Fix the refresh-rate fallback's divide-by-zero.** Evidence: `third-party/gpui-pre-windows/src/vsync.rs:64-81` converts ticks by first dividing the frequency by 1,000,000, then dividing counts by that result. The refresh-rate fallback passes rational rates such as 60/1 or 60000/1001, producing a zero divisor. A temporary harness extracted the exact helper and confirmed panics for 60 Hz, 59.94 Hz, and 144 Hz inputs while the ordinary 10 MHz QPC case succeeded. The panic can terminate the VSyncProvider thread initialized in `third-party/gpui-pre-windows/src/platform.rs:380-407`; the surrounding `Result` fallback cannot catch it. Use checked rational duration conversion and validate zero/invalid frequencies. Verify common rates, fractional rates, QPC conversion, overflow, and the DWM fallback path. A physical DWM fallback trigger was not induced on this machine.

- [ ] **WIN-002 | Medium | Investigation: verify clipboard writes use a valid owner window.** API-contract finding: `third-party/gpui-pre-windows/src/clipboard.rs:70-88,150-159,340-342` opens with `OpenClipboard(None)`, calls `EmptyClipboard`, and then calls `SetClipboardData`. [Microsoft's documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata) states that this sequence leaves a null owner and causes the write to fail. Native copy/cut round trips were not performed, so confirm the behavior with the app's search/playlist inputs before treating it as a reproduced defect. If confirmed, pass a valid application HWND, preserve correct memory ownership, and verify text copy/cut/paste plus clipboard contention without silently losing copied text. Check whether the existing backend clipboard tests cover this exact sequence.

## Review coverage and fresh checks

The ledger covers repository-owned and tracked vendored files. Source files were read through their full contents, including conditional code, error paths, and embedded tests. Generated data received format/integrity checks rather than a claim that binary bytes were source code. `.git` internals, Cargo's external dependency cache, and generated `target/` intermediates are not application source. The ignored `dist/` artifacts were inspected separately as described below; their 943 packaged dependency manifests were counted, but those external dependency sources were not individually audited.

Fresh results on native Windows:

- `cargo test --locked --offline -p tremors-core -j 2`: all 35 integration tests passed; unit/doc targets contained no tests.
- `cargo clippy --locked --offline -p tremors-core --all-targets -j 2 -- -D warnings`: passed. This does not cover desktop or vendored-backend linting.
- `cargo fmt --all --manifest-path tremors-music-app/Cargo.toml -- --check`, `node --check docs/scripts.js`, PowerShell AST parsing for all three `.ps1` scripts, and `bash -n` for the Linux script: passed.
- Cargo metadata resolved offline. The complete lockfile parsed as 946 unique package identities, including 943 registry checksums. No fresh vulnerability-database scan was performed.
- Temporary production-API harnesses reproduced CORE-001, CORE-002, CORE-003, and CORE-004. An extracted-helper harness reproduced WIN-001. These ran outside the repository and did not modify application sources.
- The installed v0.1.0 executable matched the portable executable's SHA-256 (`3d1d10d068618d73b328d09b60e880dfbabf4be145f1cdda94a4e86ef6355740`). With `TREMORS_DATA_DIR` pointing to a temporary directory, it imported all eight fixtures without scan warnings, advanced playback through the fixture queue, rendered cached artwork, and displayed ordinary/fullscreen/settings views. The log identified Direct3D 11.1 on Intel Iris Xe. Physical listening quality, output switching, scaling, screen-reader behavior, and SMTC were not certified by these observations. The review process was stopped after inspection; graceful shutdown was not a completed manual check.
- All eight audio fixtures were probed: mono 44.1 kHz PCM WAV, MP3, FLAC, Vorbis, ADTS AAC, M4A AAC, M4A ALAC, and FLAC with PNG artwork. Existing tests exercised metadata plus decoding/seek behavior for the supported fixture formats.
- Every SVG parsed, with no external references, event-handler attributes, or unresolved internal references. The root and website copies matched. PNG dimensions were 1254 x 1254; the ICO directory contained seven valid-sized frames from 16 to 256 pixels. The SVG branding/avatar rendered in the website preview. No artwork redesign was performed.
- All 18 compiled shaders matched `shaders/SHA256SUMS` and had valid DXBC lengths/chunk boundaries. Their source HLSL and Rust binding/layout paths were reviewed. Shader compilation and GPU recovery/fault injection were not rerun.
- All three existing Windows distribution checksum sidecars matched their artifacts. The source ZIP contained every tracked file, with the two branding spelling differences in BUILD-001 and the expected added Cargo vendor configuration. It contained 49,495 entries and 943 vendored dependency manifests. A fresh release build, install/upgrade/uninstall cycle, and offline rebuild from the source archive were not performed.
- The local website was inspected at desktop, 760-pixel, and 360-pixel widths with no horizontal overflow. Internal anchors resolved, lazy-loaded images loaded after scrolling, and the FAQ disclosure worked. The unavailable-release API path displayed its fallback message while download links remained available. Other GitHub API failure/pagination cases were reviewed from source, not mocked in a test suite.

### File-by-file ledger

Paths below are relative to the repository root. "Reviewed" denotes source/document inspection, not a promise that every possible runtime path is defect-free. A finding ID points to unfinished work above.

| File | Review / verification |
| --- | --- |
| `.gitattributes` | Text normalization and shell-script LF policy reviewed. |
| `.gitignore` | Build, distribution, local database/cache/log, editor, and OS exclusions reviewed. |
| `CHANGELOG.md` | Entire version history reviewed; subsequent branding fix recorded under 0.1.0. |
| `DEVELOPMENT.md` | Architecture, conventions, commands, packaging, privacy, notices, website, and manual checklists reviewed; BUILD-001. |
| `LICENSE.md` | Project declaration and all GPL sections reviewed; unchanged. |
| `PRIVACY.md` | Local files/data and user controls reviewed against desktop/source behavior. |
| `README.md` | Overview, formats, setup, links, and logo reference reviewed; BUILD-001. |
| `TASKS.md` | Existing backlog and historical verification preserved; review evidence added here. |
| `assets/TremorsMusic.ico` | All seven directory entries and image bounds checked; native window icon observed. |
| `assets/qtremors.svg` | Complete XML/reference inspection; website copy matched and rendered. |
| `assets/TremorsMusic.png` | PNG dimensions/signature and matching website copy checked; case-only rename verified; BUILD-001 resolved. |
| `assets/TremorsMusic.svg` | Complete XML/reference inspection, including gradients/filters/geometry; matching website copy rendered; case-only rename verified; BUILD-001 resolved. |
| `docs/assets/TremorsMusic.png` | Compared with canonical PNG; checked dimensions. |
| `docs/assets/TremorsMusic.svg` | Compared with canonical SVG; parsed and rendered. |
| `docs/assets/qtremors.svg` | Compared with canonical avatar; parsed and rendered. |
| `docs/index.html` | Every section, semantic element, FAQ, link, fallback, and asset reference reviewed; browser checks completed. |
| `docs/scripts.js` | Requests, timeout, pagination, totals, DOM updates, and failure fallback reviewed; syntax passed. |
| `docs/styles.css` | Every selector and breakpoint reviewed; three viewport widths checked. |
| `tremors-music-app/.cargo/config.toml` | Target-specific static CRT configuration and packaged vendor additions reviewed. |
| `tremors-music-app/Cargo.lock` | Entire lockfile parsed; identities, sources, checksums, dependency versions, and local patches inspected. |
| `tremors-music-app/Cargo.toml` | Workspace members, release profile, shared versions, and Windows patch reviewed. |
| `tremors-music-app/rust-toolchain.toml` | Pinned toolchain/components reviewed and native toolchain confirmed. |
| `tremors-music-app/crates/core/Cargo.toml` | Platform features, decoder/image formats, database, watcher, and test dependencies reviewed. |
| `tremors-music-app/crates/core/src/browsing.rs` | Search/ranking, sorts, group construction, stable keys, and album/artist ordering reviewed. |
| `tremors-music-app/crates/core/src/lib.rs` | Public module surface and platform boundaries reviewed. |
| `tremors-music-app/crates/core/src/library.rs` | Schema/migration, snapshots, roots, scans, state, playlists, ordering, relocation, reset, and errors reviewed; CORE-001/004/005. |
| `tremors-music-app/crates/core/src/lyrics.rs` | Timestamp parsing, offsets, sorting, duplicate timestamps, and active-line lookup reviewed. |
| `tremors-music-app/crates/core/src/model.rs` | All records, defaults, metadata fields, and equality semantics reviewed. |
| `tremors-music-app/crates/core/src/playback.rs` | Platform branches, command generations, loading, output, pause/resume, seeking, volume, gain, and errors reviewed. |
| `tremors-music-app/crates/core/src/queue.rs` | Replace/append/play-next, duplicates, shuffle, repeat, cursor changes, reorder, state validation, restore, and reconciliation reviewed; CORE-001. |
| `tremors-music-app/crates/core/src/scanner.rs` | Extensions, traversal, cancellation, warnings, caching, metadata/lyrics/artwork, fingerprints, and cleanup reviewed; CORE-003. |
| `tremors-music-app/crates/core/src/service.rs` | Worker lifecycle, channels, command ordering, scan events, debounce, watcher registration, and error paths reviewed; CORE-002/005. |
| `tremors-music-app/crates/core/src/settings.rs` | Preferences/defaults, sorting/repeat enums, paths, environment override, and data-directory creation reviewed. |
| `tremors-music-app/crates/core/tests/features.rs` | Every setup/assertion reviewed; passed. |
| `tremors-music-app/crates/core/tests/fixtures/tone-aac.m4a` | AAC container/stream probed; existing format tests passed. |
| `tremors-music-app/crates/core/tests/fixtures/tone-alac.m4a` | ALAC container/stream probed; existing format tests passed. |
| `tremors-music-app/crates/core/tests/fixtures/tone-art.flac` | FLAC/PNG streams probed; artwork test passed; CORE-003 reproduction fixture. |
| `tremors-music-app/crates/core/tests/fixtures/tone.aac` | ADTS AAC stream probed; existing format tests passed. |
| `tremors-music-app/crates/core/tests/fixtures/tone.flac` | FLAC stream probed; existing format/service tests passed. |
| `tremors-music-app/crates/core/tests/fixtures/tone.mp3` | MP3 stream probed; existing format tests passed. |
| `tremors-music-app/crates/core/tests/fixtures/tone.ogg` | Vorbis stream probed; existing format tests passed. |
| `tremors-music-app/crates/core/tests/fixtures/tone.wav` | PCM stream probed; existing format tests passed. |
| `tremors-music-app/crates/core/tests/formats.rs` | All metadata, decode, seek, artwork, and cache assertions reviewed; passed. |
| `tremors-music-app/crates/core/tests/library.rs` | All library fixtures, mutations, preservation, persistence, and recovery assertions reviewed; passed. |
| `tremors-music-app/crates/core/tests/queue.rs` | All queue state/cursor/shuffle/repeat/duplicate assertions reviewed; passed. |
| `tremors-music-app/crates/core/tests/service.rs` | All worker scan/cancel/error-recovery assertions reviewed; passed; watcher replacement remains CORE-002. |
| `tremors-music-app/crates/desktop/Cargo.toml` | GUI, media controls, portal/dialog features, platform dependencies, and resource build dependencies reviewed. |
| `tremors-music-app/crates/desktop/build.rs` | Resource/icon discovery and version-resource generation reviewed. |
| `tremors-music-app/crates/desktop/resources/app.rc` | Resource declarations and product/file version mapping reviewed. |
| `tremors-music-app/crates/desktop/src/main.rs` | Platform gates, release subsystem, startup, panic boundary, and exit handling reviewed. |
| `tremors-music-app/crates/desktop/src/media.rs` | SMTC/MPRIS setup, metadata/URI conversion, playback publication, and errors reviewed; UI-002. |
| `tremors-music-app/crates/desktop/src/startup.rs` | Log path/writer, panic reporting, platform error dialogs, and startup failures reviewed. |
| `tremors-music-app/crates/desktop/src/ui.rs` | Every section/function reviewed; startup, worker events, navigation, playback, shortcuts, all views/menus/overlays, and Drop; CORE-005, UI-001/002/003, architecture follow-up. |
| `tremors-music-app/installer/licenses/NSIS.txt` | Attribution inventory and all included license/exception sections reviewed. |
| `tremors-music-app/installer/windows.nsi` | Version/platform guard, install location, payload, shortcuts, registry, uninstaller, and deletion scope reviewed; lifecycle not executed. |
| `tremors-music-app/scripts/build-installer.ps1` | Tool discovery, parameters, payload/version/size checks, NSIS execution, and checksum generation reviewed; syntax passed. |
| `tremors-music-app/scripts/build-linux.sh` | Tests/build, architecture/profile handling, staging, source/vendor collection, archive/checksum, launcher, and cleanup reviewed; syntax passed; BUILD-001. |
| `tremors-music-app/scripts/build-windows.ps1` | Toolchain/profile, payload, source/vendor copying, portable/source packages, cleanup, and installer invocation reviewed; syntax passed. |
| `tremors-music-app/scripts/compile-shaders.ps1` | Compiler discovery, entry-point/profile mapping, output/error handling, and checksum generation reviewed; syntax passed. |
| `tremors-music-app/third-party/gpui-pre-windows/Cargo.toml` | All feature/dependency declarations and package metadata reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/LICENSE-APACHE` | Copyright/declaration and all Apache sections reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/build.rs` | Shader discovery, DXBC validation, generated bindings, and rebuild triggers reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/SHA256SUMS` | Every entry matched its compiled shader. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/emoji_rasterization_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/emoji_rasterization_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/monochrome_sprite_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/monochrome_sprite_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/path_rasterization_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/path_rasterization_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/path_sprite_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/path_sprite_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/polychrome_sprite_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/polychrome_sprite_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/quad_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/quad_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/shadow_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/shadow_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/subpixel_sprite_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/subpixel_sprite_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/underline_ps.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/shaders/underline_vs.cso` | DXBC length/chunks and SHA-256 checked; corresponding HLSL reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/alpha_correction.hlsl` | Gamma/contrast math, scalar/vector variants, and call sites reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/clipboard.rs` | Formats, ownership/locking, text/images/files, DIB conversion, cleanup, and tests reviewed; WIN-002. |
| `tremors-music-app/third-party/gpui-pre-windows/src/color_text_raster.hlsl` | Emoji raster/compositing buffers, gamma/contrast, and vertex/fragment paths reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/destination_list.rs` | Jump-list/COM setup, iteration, and cleanup reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/dialog.rs` | Native dialog UI, asynchronous synchronization, close/cancel paths, and callbacks reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/direct_manipulation.rs` | Touch/scroll state, callbacks, transforms, event emission, and cleanup reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/direct_write.rs` | Font selection/loading, layout/index conversion, glyph arrays/clusters, rasterization/compositing, device recovery, and embedded tests reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/directx_atlas.rs` | Texture allocation/upload, tile lifecycle, bounds, locking, and device recovery reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/directx_devices.rs` | Adapter/feature selection, software fallback, device creation, diagnostics, and recovery reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/directx_renderer.rs` | All resources/pipelines, buffers/layouts, draw batches, shader loading, resizing, readback, recovery, and vendor driver diagnostics reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/dispatcher.rs` | Foreground/background dispatch, delayed work, parking/wakeup, and thread boundaries reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/display.rs` | Monitor enumeration/identity, bounds, DPI, and screen capture paths reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/events.rs` | Every window-message branch, callbacks/reentrancy, key/mouse input, IME, theme/DPI, device loss, visibility, and helper reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/gpui_windows.rs` | Modules, exports, and Windows feature gates reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/keyboard.rs` | Layout mapping, modifiers, key names, caching, and event conversion reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/platform.rs` | Initialization, event loop, window lifecycle, task dispatch, dialogs, notifications, clipboard routing, and GPU recovery reviewed; WIN-001. |
| `tremors-music-app/third-party/gpui-pre-windows/src/shaders.hlsl` | Every struct/helper and quad/shadow/path/underline/mono/subpixel/polychrome entry point reviewed against renderer layouts/bindings. |
| `tremors-music-app/third-party/gpui-pre-windows/src/system_notifications.rs` | Notification subscription, dispatch, and lifecycle reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/system_settings.rs` | System appearance/settings queries, update routing, caches, and fallbacks reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/util.rs` | Handle wrappers, recovery/utilities, DLL lifetime, and error paths reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/vsync.rs` | DWM/QPC/rational timing, fallback/sleep behavior, and thread callers reviewed; WIN-001 reproduced. |
| `tremors-music-app/third-party/gpui-pre-windows/src/window.rs` | Construction, state, focus/input, bounds/fullscreen, rendering coordination, cursor, controls, platform trait methods, and destruction reviewed. |
| `tremors-music-app/third-party/gpui-pre-windows/src/wrapper.rs` | Platform wrapper forwarding and trait boundaries reviewed. |

## Verification baseline

Recorded on 2026-10-10. These results describe completed checks, rather than a fresh release approval:

- All 35 core integration tests passed on native Windows MSVC; Rust formatting passed.
- The native Windows MSVC release and NSIS packaging completed, with matching executable versions and SHA-256 sidecars.
- Separate binary/source packaging and checksums passed on Windows with actual NSIS compilation and stubbed compiler/vendor operations. Linux x64, ARM64, and debug packaging also passed with compiler/vendor operations stubbed. These establish packaging behavior, rather than native Linux runtime results.
- The documented Linux user installation, launcher paths, and uninstall commands passed in an isolated directory using a packaging fixture.
- Native Windows rendering, physical audio, SMTC, installer lifecycle, Linux Wayland, hardware rendering, desktop portal behavior, and large-library performance remain outstanding.

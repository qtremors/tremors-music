# Tremors Music Changelog

These balanced source checkpoints were reconstructed on 2026-10-20.
The complete desktop application and locked dependency graph are assembled at 0.1.0.

## [0.0.2] - 2026-10-20

- **Music Scanning:** Added local metadata extraction, supported-format detection, and embedded artwork caching.
- **Incremental Updates:** Added file and lyric-sidecar fingerprints to skip unchanged tracks while preserving existing user data.
- **Scan Safety:** Added cancellation, traversal warnings, and safeguards against pruning records after incomplete scans.
- **Format Coverage:** Added original audio fixtures and checks for decoding, seeking, metadata, and reusable artwork.

## [0.0.1] - 2026-10-20

- **Rust Foundation:** Established the Rust 2024 workspace, pinned toolchain, shared models, and desktop scaffold.
- **Local Library:** Added SQLite storage, local data paths, preferences, favorites, play counts, and playlist persistence.
- **Data Integrity:** Preserved duplicate playlist entries and added persistence regression tests.

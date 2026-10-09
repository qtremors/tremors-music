# Tremors Music - Releases

> **Project:** Tremors Music
> **Version:** 0.1.0
> **Last Updated:** 2026-10-20

| Version | Key Focus |
| --- | --- |
| [v0.1.0](#v010) | First release: native desktop playback, local library, playlists, and lyrics |

---

# v0.1.0

Tremors Music v0.1.0 is the first release of the native Rust and GPUI application for Windows and Linux. It gives your local music collection a home without accounts, ads, telemetry, or cloud services.

## Highlights

- Browse songs, albums, artists, genres, favorites, recently added, and most played.
- Build playlists and reorder your playback queue, including duplicate tracks.
- Follow local timed lyrics and select a line to seek.
- Restore your queue and position after reopening, with playback paused.
- Choose light or dark themes, seven accents, and fullscreen playback.
- Keep music, lyrics, preferences, and library data on your device.

## Music Library and Folders

- **Search and sorting:** Find tracks across your library and browse album and artist collections.
- **Folder management:** Add folders through a native picker or typed path, rescan, cancel scans, and relocate a folder while retaining matching library records.
- **Incremental updates:** Skip unchanged files and optionally watch registered folders for changes.
- **Local maintenance:** Remove folders, reset library records, and clean unused cached artwork without editing or deleting music and lyric files.

## Playback, Queues, and Playlists

- **Transport controls:** Play, pause, seek, change volume, shuffle, and repeat, with keyboard shortcuts and desktop media controls through Windows SMTC or Linux MPRIS.
- **Queue editing:** Play next, append songs, and reorder entries with buttons or drag-and-drop while preserving duplicate entries.
- **Saved listening:** Create, rename, and delete playlists. Save the queue and playback position for a paused return on the next launch.
- **ReplayGain:** Enable optional track normalization with peak limiting.
- **Audio formats:** Play MP3, FLAC, PCM WAV, Ogg Vorbis, AAC/ADTS, and AAC or ALAC in M4A. WMA and Opus are unsupported.

## Lyrics and Appearance

- **Local lyrics:** Read embedded plain lyrics, LRC, and millisecond ID3 SYLT. A matching UTF-8 `.lrc` sidecar takes precedence.
- **Timed playback:** Follow highlighted lines, scroll with playback, and seek by selecting a lyric line.
- **Your listening space:** Switch themes and accents, control song-list artwork visibility, and open an immersive fullscreen player.

## Privacy and Desktop Setup

- **Offline use:** No sign-in, uploads, cloud sync, online metadata lookups, or automatic update checks. Startup diagnostics remain local.
- **Windows:** Requires Windows 10 version 1903 or newer, or Windows 11, on x64. Packaging supports a per-user installer and a portable ZIP.
- **Linux:** Native builds use the host's graphics, audio, and font libraries. Build and installation guidance covers Debian/Ubuntu, Arch, and Fedora.
- **Source and integrity:** Packaging includes separate matching source archives, dependency notices, and SHA-256 checksums. The application is licensed under GNU GPL v3 only.

## Getting Started and Support

See the [README](README.md#getting-started) to add your music and learn the controls. Download available packages from [GitHub Releases](https://github.com/qtremors/tremors-music/releases), and report problems through [GitHub Issues](https://github.com/qtremors/tremors-music/issues).

Read the [privacy policy](PRIVACY.md) for local storage and reset behavior, the [developer guide](DEVELOPMENT.md) for builds and verification, and [TASKS.md](TASKS.md) for open findings and remaining platform checks. Version-specific development history is recorded in [CHANGELOG.md](CHANGELOG.md).

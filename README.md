<p align="center">
  <img src="assets/TremorsMusic.svg" alt="Tremors Music logo" width="120" />
</p>

<h1 align="center"><a href="https://qtremors.github.io/tremors-music/">Tremors Music</a></h1>

<p align="center">Your music. Your space.</p>
<p align="center">A private, native music player for Windows and Linux.</p>

<p align="center">
  <a href="https://github.com/qtremors/tremors-music/releases/latest">
    <img src="https://img.shields.io/github/v/release/qtremors/tremors-music?label=Download%20EXE&amp;color=7947bd&amp;logo=github&amp;logoColor=white" alt="Download EXE" height="32" />
  </a>
</p>

<p align="center">
  <a href="https://github.com/qtremors/tremors-music/releases"><img src="https://img.shields.io/github/downloads/qtremors/tremors-music/total?label=Total%20Downloads&amp;color=538bcc" alt="Total Downloads" /></a>
  <a href="https://github.com/qtremors/tremors-music/releases"><img src="https://img.shields.io/github/downloads/qtremors/tremors-music/latest/total?label=Latest%20Downloads&amp;color=c02860" alt="Latest Downloads" /></a>
</p>
<p align="center">
  <img src="https://img.shields.io/badge/Platforms-Windows%20%7C%20Linux-538bcc" alt="Windows and Linux" />
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/License-GPL%203.0%20only-7947bd" alt="GPL v3 only" /></a>
</p>

> [!NOTE]
> **Privacy first:** Music, lyrics, playlists, and preferences stay on your device. The desktop app has no accounts, ads, telemetry, or online metadata lookups.

## Why Tremors Music

Tremors Music gives your local music collection a home without accounts, ads, telemetry, or cloud services. Your library stays on your device. Music files change only when you save edits in the metadata editor.

## Download

Find packages on [GitHub Releases](https://github.com/qtremors/tremors-music/releases), or build from source using the [developer guide](DEVELOPMENT.md). Open Music folders, add a folder, and choose a track after scanning.

Windows builds include a per-user installer and a portable ZIP, with corresponding source as a separate download. Windows requires Windows 10 version 1903 or newer, or Windows 11, on x64. The currently prepared release files are Windows-only. For Linux, follow the [build instructions for Debian/Ubuntu, Arch, and Fedora](DEVELOPMENT.md#linux), then the [installation guide](DEVELOPMENT.md#linux-installation). Linux builds are native tar archives and need compatible graphics, audio, and font libraries.

## Features

- **Library home:** Rediscover recently added albums, records in your rotation, and your own playlists, all drawn from music on your device.
- **Local library:** Browse artwork galleries for albums, artists, genres, and playlists. Open album details, switch between an artist’s albums and songs, or use compact song tables with global search and sorting.
- **Desktop layout:** Keep playback, progress, volume, favorites, lyrics, and Up next within reach in the persistent top player. Use the grouped sidebar to move through your library.
- **Music folders:** Choose folders with a native picker or typed paths. Scan incrementally, watch for changes, cancel scans, and relocate folders while retaining matching records.
- **Playback:** Play, pause, seek, shuffle, repeat, adjust volume, and use desktop media controls on Windows and Linux.
- **Queues and playlists:** Play next, add and reorder tracks, preserve duplicate entries, and create, rename, or delete playlists.
- **Resume listening:** Save your queue and position. Reopening restores playback paused.
- **Lyrics:** Read embedded plain lyrics, LRC, and millisecond ID3 SYLT, or matching local `.lrc` files. Follow timed lyrics and select a line to seek.
- **Personalization:** Light, dark, and true-black OLED themes, seven accents, reduced motion, artwork visibility, fullscreen playback, and keyboard shortcuts.
- **Metadata editing:** Edit standard and extended tags, embedded lyrics, and artwork in each file’s supported format. Saving validates a temporary copy before replacing the original; library identity and curation are retained.
- **Sleep timer:** Choose 5–90 minutes, an optional final fade, and whether to finish the current track.
- **Portable playlists:** Import UTF-8 M3U/M3U8 files and export playlists with relative paths, preserving order and repeated tracks.
- **Library tools:** Recently played tracks, local folder covers, custom artist and playlist images, folder exclusions, and a minimum-duration filter.
- **ReplayGain:** Optional track normalization with peak limiting.

Supported formats are MP3, FLAC, PCM WAV, Ogg Vorbis, AAC/ADTS, and AAC or ALAC in M4A. WMA and Opus are not supported.

## Getting started

1. Open the Windows shortcut after installation, or extract the portable ZIP and run `tremors-music.exe`. On Linux, run `tremors-music`.
2. Open **Music folders** and add a folder with the native picker or a typed path.
3. Let the scan finish, then choose a track in Songs, Albums, or Artists.

Use the **+** beside **Playlists** to create a mix, and the playlist’s **…** menu to rename or delete it. Use Play next, and reorder your queue with buttons or drag-and-drop. The queue and position are saved; reopening restores playback paused. Folder watching picks up changes automatically and can be disabled. Manual rescans, scan cancellation, and folder relocation are available in Music folders.

A matching UTF-8 `.lrc` file takes precedence over embedded lyrics. Timed lyrics highlight during playback; select a line to seek. Sidecar lyric files remain unchanged. To edit embedded tags, lyrics, or artwork, use **Edit metadata** in a track’s menu and choose **Save to file**. Music folders displays the local data directory; see [PRIVACY.md](PRIVACY.md) for storage and reset behavior.

Choose **OLED** in **Settings → Appearance** for true-black surfaces. **Reduce motion** disables entry transitions. Open the clock button in the player for the sleep timer. Use **Import M3U** on All playlists and **Export M3U** within a playlist. Missing or unsupported import entries are reported under Music folders; valid entries retain their order. Exported relative paths require the music files to move with the playlist.

Music folders includes non-destructive duration and folder filters. Scans prefer embedded covers, then `cover.jpg`, `cover.png`, `folder.jpg`, or `folder.png` beside the music. Use **Artwork** on artist and playlist pages for a local image override.

### Keyboard shortcuts

| Shortcut | Action |
| --- | --- |
| Space | Play / pause |
| Left / Right, or Ctrl + Left / Right | Previous / next track |
| Shift + Left / Right | Seek backward / forward 10 seconds |
| Up / Down | Raise / lower volume |
| Ctrl + M | Mute / unmute |
| Ctrl + K, or / | Focus search |
| Ctrl + Q | Show queue |
| Ctrl + L | Show lyrics |
| Ctrl + F, or F11 | Toggle fullscreen player |
| Alt + Left | Go back |
| Escape | Close an overlay / leave fullscreen |

Playback shortcuts are disabled while editing text. Ctrl + K, F11, and Escape remain available in inputs.

## Privacy

The desktop app processes music locally, with no accounts, telemetry, uploads, cloud sync, online metadata lookups, or automatic update checks. Music files change only when you explicitly save metadata; sidecar lyrics remain unchanged.

Read [PRIVACY.md](PRIVACY.md) for stored data, local diagnostics, desktop integration, backup and removal, and the separate requests made by the project website.

## Community and support

- [Project website](https://qtremors.github.io/tremors-music/).
- [GitHub Issues](https://github.com/qtremors/tremors-music/issues) for bugs and feature requests.
- [Release notes](RELEASES.md), [changelog](CHANGELOG.md), and [privacy policy](PRIVACY.md).

## Credits

Tremors Music is built by [Tremors](https://github.com/qtremors) with Rust and native desktop tools. Thanks to the maintainers of:

- [Rust](https://www.rust-lang.org/), [GPUI Kit](https://docs.rs/gpui-kit/) and GPUI for the desktop application.
- [SQLite](https://www.sqlite.org/) and [Rusqlite](https://docs.rs/rusqlite/) for local persistence.
- [Rodio](https://docs.rs/rodio/), [Symphonia](https://docs.rs/symphonia/) and [CPAL](https://docs.rs/cpal/) for playback and decoding.
- [Lofty](https://docs.rs/lofty/), [Notify](https://docs.rs/notify/), [Souvlaki](https://docs.rs/souvlaki/) and [Serde](https://serde.rs/) for metadata, folder watching, media controls and state serialization.

The website uses HTML, CSS, JavaScript, system fonts, and the project's own branding. Each dependency retains its own license. Matching source archives include dependency sources and notices; renderer and installer details are in the [third-party notices](DEVELOPMENT.md#third-party-notices).

## For developers

The Rust workspace lives in `tremors-music-app/`, shared branding in `assets/`, and the static website in `docs/`. Architecture, setup, naming, builds, tests, distribution, and specialist documentation are described in [DEVELOPMENT.md](DEVELOPMENT.md).

## License

Tremors Music is licensed under [GNU GPL v3 only](LICENSE.md). That document includes the full license text. Distributed derivatives must preserve notices and provide corresponding source under the GPL.

<p align="center">Made by <a href="https://github.com/qtremors">Tremors</a></p>

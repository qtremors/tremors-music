# Architecture Guide: Tremors Music

```
========================================================================
Language                    Files        Blank      Comment         Code
------------------------------------------------------------------------
JSON                           10            5            0         5047
TypeScript TSX                 36          518           87         4886
Python                         10          177          140          908
TypeScript                     11          129           80          750
Markdown                        3          108            0          328
JavaScript                      3            1            1           51
TOML                            2            4            1           37
CSS                             1            6            2           28
Rust                            3            3            2           28
HTML                            1            0            0           13
------------------------------------------------------------------------
TOTAL                          80          951          313        12076
========================================================================
```


```
tremors-music/
├── backend/
│   ├── backend/
│   │   └── .art/
│   │       └── 35.jpg
│   ├── router/
│   │   ├── library.py
│   │   ├── media.py
│   │   ├── playlists.py
│   │   └── stream.py
│   ├── .art
│   ├── database.py
│   ├── main.py
│   ├── models.py
│   ├── pyproject.toml
│   ├── scanner.py
│   ├── scanner_progress.py
│   ├── streamer.py
│   └── uv.lock
├── frontend/
│   ├── public/
│   │   ├── music.svg
│   │   └── vite.svg
│   ├── src/
│   │   ├── assets/
│   │   │   └── react.svg
│   │   ├── components/
│   │   │   ├── common/
│   │   │   │   ├── Button.tsx
│   │   │   │   ├── Card.tsx
│   │   │   │   └── IconButton.tsx
│   │   │   ├── player/
│   │   │   │   ├── QueuePanel.tsx
│   │   │   │   └── SyncedLyrics.tsx
│   │   │   ├── settings/
│   │   │   │   ├── DangerZone.tsx
│   │   │   │   ├── LibraryPathManager.tsx
│   │   │   │   ├── ScannerControl.tsx
│   │   │   │   └── ThemeSettings.tsx
│   │   │   ├── AddToPlaylistModal.tsx
│   │   │   ├── ConfirmDialog.tsx
│   │   │   ├── ContextMenu.tsx
│   │   │   ├── CreatePlaylistModal.tsx
│   │   │   ├── ErrorBoundary.tsx
│   │   │   ├── FullScreenPlayer.tsx
│   │   │   ├── PageWrapper.tsx
│   │   │   ├── Player.tsx
│   │   │   ├── SettingsModal.tsx
│   │   │   ├── Skeletons.tsx
│   │   │   ├── SongList.tsx
│   │   │   ├── SquigglyProgress.tsx
│   │   │   └── ToastContainer.tsx
│   │   ├── hooks/
│   │   │   └── useKeyboardShortcuts.ts
│   │   ├── lib/
│   │   │   ├── animations.ts
│   │   │   ├── api.ts
│   │   │   └── utils.ts
│   │   ├── pages/
│   │   │   ├── AlbumDetail.tsx
│   │   │   ├── AlbumsPage.tsx
│   │   │   ├── AllPlaylistsPage.tsx
│   │   │   ├── ArtistDetail.tsx
│   │   │   ├── ArtistsPage.tsx
│   │   │   ├── GenreDetail.tsx
│   │   │   ├── GenresPage.tsx
│   │   │   ├── LibraryPage.tsx
│   │   │   ├── PlaylistDetail.tsx
│   │   │   ├── SearchPage.tsx
│   │   │   ├── SettingsPage.tsx
│   │   │   └── SmartPlaylistDetail.tsx
│   │   ├── stores/
│   │   │   ├── confirmStore.ts
│   │   │   ├── filterStore.ts
│   │   │   ├── playerStore.ts
│   │   │   ├── themeStore.ts
│   │   │   └── toastStore.ts
│   │   ├── App.tsx
│   │   ├── index.css
│   │   ├── main.tsx
│   │   └── types.ts
│   ├── eslint-report.json
│   ├── eslint.config.js
│   ├── index.html
│   ├── package-lock.json
│   ├── package.json
│   ├── postcss.config.js
│   ├── tailwind.config.js
│   ├── tsconfig.app.json
│   ├── tsconfig.json
│   ├── tsconfig.node.json
│   └── vite.config.ts
├── src-tauri/
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── 32x32.png
│   │   ├── icon.icns
│   │   ├── icon.ico
│   │   ├── icon.png
│   │   ├── Square107x107Logo.png
│   │   ├── Square142x142Logo.png
│   │   ├── Square150x150Logo.png
│   │   ├── Square284x284Logo.png
│   │   ├── Square30x30Logo.png
│   │   ├── Square310x310Logo.png
│   │   ├── Square44x44Logo.png
│   │   ├── Square71x71Logo.png
│   │   ├── Square89x89Logo.png
│   │   └── StoreLogo.png
│   ├── src/
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── .gitignore
│   ├── binaries
│   ├── build.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .git/
├── .gitattributes
├── .gitignore
├── ARCHITECTURE.md
├── CHANGELOG.md
├── package-lock.json
├── package.json
└── README.md
```
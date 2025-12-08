# Architecture Guide: Tremors Music

```
========================================================================
Language                    Files        Blank      Comment         Code
------------------------------------------------------------------------
JSON                           10            2            0         5074
TypeScript TSX                 36          542          100         5056
Python                         12          224          200         1113
TypeScript                     11          136           86          790
Markdown                        5          147            0          636
Rust                            3            5            3           52
JavaScript                      3            1            1           51
TOML                            2            5            1           42
CSS                             1            6            2           28
HTML                            1            0            0           13
XML                             2            0            0            9
------------------------------------------------------------------------
TOTAL                          86         1068          393        12864
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
│   ├── backend.spec
│   ├── backend_build.py
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
├── scripts/
│   └── build-backend.mjs
├── src-tauri/
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   │   ├── android/
│   │   │   ├── mipmap-anydpi-v26/
│   │   │   │   └── ic_launcher.xml
│   │   │   ├── mipmap-hdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   ├── mipmap-mdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   ├── mipmap-xhdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   ├── mipmap-xxhdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   ├── mipmap-xxxhdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   └── values/
│   │   │       └── ic_launcher_background.xml
│   │   ├── ios/
│   │   │   ├── AppIcon-20x20@1x.png
│   │   │   ├── AppIcon-20x20@2x-1.png
│   │   │   ├── AppIcon-20x20@2x.png
│   │   │   ├── AppIcon-20x20@3x.png
│   │   │   ├── AppIcon-29x29@1x.png
│   │   │   ├── AppIcon-29x29@2x-1.png
│   │   │   ├── AppIcon-29x29@2x.png
│   │   │   ├── AppIcon-29x29@3x.png
│   │   │   ├── AppIcon-40x40@1x.png
│   │   │   ├── AppIcon-40x40@2x-1.png
│   │   │   ├── AppIcon-40x40@2x.png
│   │   │   ├── AppIcon-40x40@3x.png
│   │   │   ├── AppIcon-512@2x.png
│   │   │   ├── AppIcon-60x60@2x.png
│   │   │   ├── AppIcon-60x60@3x.png
│   │   │   ├── AppIcon-76x76@1x.png
│   │   │   ├── AppIcon-76x76@2x.png
│   │   │   └── AppIcon-83.5x83.5@2x.png
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── 32x32.png
│   │   ├── 64x64.png
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
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── gen
│   └── tauri.conf.json
├── .git/
├── .gitattributes
├── .gitignore
├── ARCHITECTURE.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE
├── package-lock.json
├── package.json
├── PRIVACY.md
├── README.md
├── tremorsmusic.png
└── tremorsmusic.svg
```
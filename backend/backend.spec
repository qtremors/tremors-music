# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller spec file for Tremors Music backend.
Creates a single-file executable with all dependencies bundled.
"""

a = Analysis(
    ['backend_build.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[
        'uvicorn.logging',
        'uvicorn.loops',
        'uvicorn.loops.auto',
        'uvicorn.protocols',
        'uvicorn.protocols.http',
        'uvicorn.protocols.http.auto',
        'uvicorn.protocols.websockets',
        'uvicorn.protocols.websockets.auto',
        'uvicorn.lifespan',
        'uvicorn.lifespan.on',
        'uvicorn.lifespan.off',
        'sqlmodel',
        'sqlalchemy.dialects.sqlite',
        'aiofiles',
        'multipart',
        'fastapi',
        'starlette',
        'pydantic',
        'mutagen',
        'watchdog',
        'watchdog.observers',
        'watchdog.events',
        # App modules
        'main',
        'database',
        'models',
        'scanner',
        'scanner_progress',
        'streamer',
        'router',
        'router.library',
        'router.stream',
        'router.media',
        'router.playlists',
        'PIL',
        'PIL.Image',
        'PIL.JpegImagePlugin',
        'PIL.PngImagePlugin',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name='tremorsmusic',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,  # Hide console window in production
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon='../tauri/src-tauri/icons/icon.ico',  # Add icon
)


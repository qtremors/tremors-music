from typing import Optional, List
from sqlmodel import SQLModel, Field, Relationship

# --- LINK TABLE ---
class PlaylistSong(SQLModel, table=True):
    playlist_id: Optional[int] = Field(default=None, foreign_key="playlist.id", primary_key=True)
    song_id: Optional[int] = Field(default=None, foreign_key="song.id", primary_key=True)
    order: int = Field(default=0)

# --- MODELS ---
class LibraryPath(SQLModel, table=True):
    id: Optional[int] = Field(default=None, primary_key=True)
    path: str = Field(unique=True, index=True)

class Album(SQLModel, table=True):
    id: Optional[int] = Field(default=None, primary_key=True)
    title: str = Field(index=True)
    artist: str = Field(index=True)
    cover_path: Optional[str] = None
    
    # --- EXPANDED ALBUM METADATA ---
    year: Optional[int] = None
    genre: Optional[str] = None
    total_tracks: Optional[int] = None
    total_discs: Optional[int] = None
    compilation: bool = Field(default=False)
    label: Optional[str] = None
    barcode: Optional[str] = None
    catalog_number: Optional[str] = None
    
    songs: List["Song"] = Relationship(back_populates="album")

class Playlist(SQLModel, table=True):
    id: Optional[int] = Field(default=None, primary_key=True)
    name: str
    songs: List["Song"] = Relationship(back_populates="playlists", link_model=PlaylistSong)

class Song(SQLModel, table=True):
    id: Optional[int] = Field(default=None, primary_key=True)
    
    # --- BASIC INFORMATION ---
    title: str = Field(index=True)
    artist: str = Field(index=True)
    album_id: Optional[int] = Field(default=None, foreign_key="album.id")
    path: str = Field(unique=True, index=True)
    
    # --- PEOPLE & CREDITS ---
    composer: Optional[str] = None
    conductor: Optional[str] = None
    lyricist: Optional[str] = None
    arranger: Optional[str] = None
    performer: Optional[str] = None
    remixer: Optional[str] = None
    engineer: Optional[str] = None
    producer: Optional[str] = None
    
    # --- ORGANIZATION & CATALOGING ---
    track_number: Optional[int] = None
    disc_number: Optional[int] = None
    genre: Optional[str] = None
    compilation: bool = Field(default=False)
    isrc: Optional[str] = None  # International Standard Recording Code
    
    # --- DATES ---
    year: Optional[int] = None
    release_date: Optional[str] = None  # YYYY-MM-DD format
    original_date: Optional[str] = None
    
    # --- TECHNICAL AUDIO INFO ---
    duration: float = Field(default=0.0)
    file_size: int = Field(default=0)  # Bytes
    bitrate: Optional[int] = None  # kbps
    sample_rate: Optional[int] = None  # Hz
    channels: Optional[int] = None  # 1=mono, 2=stereo, 6=5.1
    bits_per_sample: Optional[int] = None  # 16, 24, 32
    format: str = Field(default="mp3")  # mp3, flac, etc.
    codec: Optional[str] = None  # Codec name
    
    # --- CONTENT & DESCRIPTION ---
    has_lyrics: bool = Field(default=False)
    lyrics: Optional[str] = None  # Full lyrics text
    comment: Optional[str] = None
    description: Optional[str] = None
    language: Optional[str] = None  # ISO 639-2 code
    mood: Optional[str] = None
    
    # --- MUSICAL INFORMATION ---
    bpm: Optional[int] = None  # Beats per minute
    initial_key: Optional[str] = None  # Musical key (C, Am, etc.)
    
    # --- REPLAY GAIN (for volume normalization) ---
    replaygain_track_gain: Optional[float] = None
    replaygain_track_peak: Optional[float] = None
    replaygain_album_gain: Optional[float] = None
    replaygain_album_peak: Optional[float] = None
    
    # --- USER DATA (local only, not from tags) ---
    rating: Optional[int] = None  # 0-5 scale
    play_count: int = Field(default=0)
    last_played: Optional[str] = None  # ISO datetime
    date_added: Optional[str] = None  # ISO datetime
    
    # --- MEDIA TYPE (for podcasts, audiobooks, etc.) ---
    media_type: Optional[str] = None  # song, podcast, audiobook, etc.
    grouping: Optional[str] = None  # Content group
    subtitle: Optional[str] = None
    
    # --- RELATIONSHIPS ---
    album: Optional[Album] = Relationship(back_populates="songs")
    playlists: List[Playlist] = Relationship(back_populates="songs", link_model=PlaylistSong)
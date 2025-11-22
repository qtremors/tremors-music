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
    songs: List["Song"] = Relationship(back_populates="album")

class Playlist(SQLModel, table=True):
    id: Optional[int] = Field(default=None, primary_key=True)
    name: str
    songs: List["Song"] = Relationship(back_populates="playlists", link_model=PlaylistSong)

class Song(SQLModel, table=True):
    id: Optional[int] = Field(default=None, primary_key=True)
    title: str = Field(index=True)
    artist: str = Field(index=True)
    album_id: Optional[int] = Field(default=None, foreign_key="album.id")
    path: str = Field(unique=True, index=True)
    duration: float = Field(default=0.0)
    track_number: Optional[int] = None
    genre: Optional[str] = None
    has_lyrics: bool = Field(default=False)
    
    # --- NEW TECH SPECS ---
    file_size: int = Field(default=0)      # Bytes
    bitrate: Optional[int] = None          # kbps (e.g. 320)
    sample_rate: Optional[int] = None      # Hz (e.g. 44100)
    format: str = Field(default="mp3")     # mp3, flac, etc.

    album: Optional[Album] = Relationship(back_populates="songs")
    playlists: List[Playlist] = Relationship(back_populates="songs", link_model=PlaylistSong)
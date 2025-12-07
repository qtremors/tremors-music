import os
from fastapi import APIRouter, HTTPException
from fastapi.responses import FileResponse, Response
from sqlmodel import Session
from database import get_session, engine
from models import Album
from mutagen import File as MutagenFile
from pathlib import Path

router = APIRouter(prefix="/media", tags=["Media"])

# Create .art cache directory
ART_CACHE_DIR = Path(".art")
ART_CACHE_DIR.mkdir(exist_ok=True)

def get_cached_art_path(album_id: int) -> Path:
    """Get the path to cached album art."""
    return ART_CACHE_DIR / f"{album_id}.jpg"

def extract_and_cache_album_art(album: Album) -> Path | None:
    """Extract album art from the first song and cache it."""
    cache_path = get_cached_art_path(album.id)
    
    # Return cached version if it exists
    if cache_path.exists():
        return cache_path
    
    # Find a song from this album to extract art from
    with Session(engine) as session:
        from models import Song
        from sqlmodel import select
        song = session.exec(
            select(Song).where(Song.album_id == album.id).limit(1)
        ).first()
        
        if not song or not os.path.exists(song.path):
            return None
        
        try:
            audio = MutagenFile(song.path)
            if not audio:
                return None
            
            # Extract cover art based on file type
            cover_data = None
            
            # MP3/ID3
            if hasattr(audio, 'tags') and audio.tags:
                for key in audio.tags.keys():
                    if key.startswith('APIC'):
                        cover_data = audio.tags[key].data
                        break
            
            # MP4/M4A
            elif hasattr(audio, 'get') and 'covr' in audio:
                cover_data = bytes(audio['covr'][0])
            
            # FLAC
            elif hasattr(audio, 'pictures') and audio.pictures:
                cover_data = audio.pictures[0].data
            
            if cover_data:
                # Save to cache
                cache_path.write_bytes(cover_data)
                return cache_path
                
        except Exception as e:
            print(f"Error extracting art for album {album.id}: {e}")
            return None
    
    return None

@router.get("/cover/{album_id}")
def get_album_cover(album_id: int):
    """Get album cover art (cached or extracted)."""
    with Session(engine) as session:
        album = session.get(Album, album_id)
        if not album:
            raise HTTPException(status_code=404, detail="Album not found")
        
        # Try to get cached or extract art
        art_path = extract_and_cache_album_art(album)
        
        if art_path and art_path.exists():
            return FileResponse(
                art_path,
                media_type="image/jpeg",
                headers={"Cache-Control": "public, max-age=31536000"}  # Cache for 1 year
            )
        
        # Return placeholder if no art found
        raise HTTPException(status_code=404, detail="No cover art found")
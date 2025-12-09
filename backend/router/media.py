from fastapi import APIRouter, Depends, Response
from sqlmodel import Session, select
from database import get_session, get_app_dir
from models import Album, Song
from mutagen import File as MutagenFile
from mutagen.id3 import ID3, APIC
from mutagen.flac import FLAC, Picture
from mutagen.mp4 import MP4
import os
import base64
import io
from PIL import Image

router = APIRouter(tags=["Media"])

# 1x1 Transparent PNG pixel
DEFAULT_COVER = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=")

# Define cache directory
COVERS_DIR = os.path.join(get_app_dir(), "covers")
os.makedirs(COVERS_DIR, exist_ok=True)

@router.get("/covers/{album_id}")
def get_album_cover(album_id: int, size: str = "small", session: Session = Depends(get_session)):
    # Validate size
    is_full = size == "full"
    
    # 1. Check Cache
    filename = f"{album_id}_full.jpg" if is_full else f"{album_id}.jpg"
    cache_path = os.path.join(COVERS_DIR, filename)
    
    if os.path.exists(cache_path):
        with open(cache_path, "rb") as f:
            return Response(content=f.read(), media_type="image/jpeg")

    album = session.get(Album, album_id)
    if not album: 
        return Response(content=DEFAULT_COVER, media_type="image/png")
    
    # Check first 3 songs for art
    songs = session.exec(select(Song).where(Song.album_id == album_id).limit(3)).all()
    
    image_data = None
    
    for song in songs:
        if not os.path.exists(song.path): continue
        try:
            audio = MutagenFile(song.path)
            if audio is None:
                continue
            
            # MP3 with ID3 tags
            if isinstance(audio, ID3) or hasattr(audio, 'tags') and isinstance(audio.tags, ID3):
                for key in audio.tags.keys():
                    if key.startswith('APIC'):
                        image_data = audio.tags[key].data
                        break
            
            # FLAC
            elif isinstance(audio, FLAC) and audio.pictures:
                image_data = audio.pictures[0].data
            
            # MP4/M4A
            elif isinstance(audio, MP4) and 'covr' in audio.tags:
                image_data = bytes(audio.tags['covr'][0])
            
            if image_data:
                break # Found art
        except:
            continue

    if image_data:
        try:
            # Resize Logic
            img = Image.open(io.BytesIO(image_data))
            
            # Convert to RGB (in case of PNG/RGBA)
            if img.mode in ('RGBA', 'P'): 
                img = img.convert('RGB')
            
            # Resize (Lanczos is high quality)
            target_size = (1200, 1200) if is_full else (300, 300)
            img.thumbnail(target_size, Image.Resampling.LANCZOS)
            
            # Save to buffer for serving
            out_io = io.BytesIO()
            img.save(out_io, format='JPEG', quality=85)
            resized_bytes = out_io.getvalue()
            
            # Try to cache to disk (Best Effort)
            try:
                with open(cache_path, "wb") as f:
                    f.write(resized_bytes)
            except Exception as e:
                print(f"[WARNING] Failed to write cache for album {album_id}: {e}")
            
            return Response(content=resized_bytes, media_type="image/jpeg")
            
        except Exception as e:
            print(f"[ERROR] Error processing image for album {album_id}: {e}")
            # Fallback to original data
            return Response(content=image_data, media_type="image/jpeg")

    # Return silent placeholder instead of 404 to fix console warnings
    return Response(content=DEFAULT_COVER, media_type="image/png")

@router.get("/lyrics/{song_id}")
def get_lyrics(song_id: int, session: Session = Depends(get_session)):
    """
    Get lyrics for a song. Lyrics are sourced from:
    1. Database cache (extracted during library scan)
    2. Embedded tags in the audio file (fallback/on-demand extraction)
    
    No internet requests are made - fully offline.
    """
    song = session.get(Song, song_id)
    if not song:
        return Response(status_code=404)
    
    # 1. Check DB Cache (populated during library scan)
    if song.synced_lyrics:
        return {"plainLyrics": song.lyrics, "syncedLyrics": song.synced_lyrics}
    if song.lyrics:
        return {"plainLyrics": song.lyrics, "syncedLyrics": None}

    # 2. Try extracting from file (fallback for unscanned files or DB miss)
    if song.path and os.path.exists(song.path):
        try:
            audio = MutagenFile(song.path, easy=True)
            if audio:
                raw_lyrics = None
                
                # Get lyrics from tags (maps to USLT for ID3)
                if 'lyrics' in audio:
                    lyrics_value = audio['lyrics']
                    raw_lyrics = lyrics_value[0] if isinstance(lyrics_value, list) else lyrics_value
                
                # Fallback: Check for USLT explicitly if 'lyrics' missing (common in ID3)
                if not raw_lyrics:
                    try:
                        # Re-open without easy=True to access raw tags
                        audio_raw = MutagenFile(song.path)
                        if isinstance(audio_raw, ID3) or (hasattr(audio_raw, 'tags') and isinstance(audio_raw.tags, ID3)):
                            # Check for USLT (Unsynced Lyrics)
                            uslt_keys = [k for k in audio_raw.keys() if k.startswith('USLT')]
                            if uslt_keys:
                                # mutagen USLT frame has .text attribute
                                raw_lyrics = getattr(audio_raw[uslt_keys[0]], 'text', str(audio_raw[uslt_keys[0]]))

                            # Check for TXXX:LYRICS (sometimes used)
                            if not raw_lyrics:
                                txxx_keys = [k for k in audio_raw.keys() if k.startswith('TXXX:LYRICS')]
                                if txxx_keys:
                                    raw_lyrics = str(audio_raw[txxx_keys[0]])
                    except:
                        pass

                if raw_lyrics and len(raw_lyrics.strip()) > 0:
                    raw_lyrics = raw_lyrics.strip()
                    
                    # Detect if synced (contains LRC timestamps like [00:00.00])
                    is_synced = '[' in raw_lyrics and ']' in raw_lyrics
                    
                    # Update DB for future requests
                    song.lyrics = raw_lyrics
                    song.has_lyrics = True
                    if is_synced:
                        song.synced_lyrics = raw_lyrics
                    
                    session.add(song)
                    session.commit()
                    
                    return {
                        "plainLyrics": raw_lyrics, 
                        "syncedLyrics": raw_lyrics if is_synced else None
                    }
        except Exception as e:
            # Log but don't fail - just means no lyrics available
            print(f"[WARNING] Error extracting lyrics for song {song_id}: {e}")

    # 3. No lyrics found - return empty response (not an error)
    return {"plainLyrics": None, "syncedLyrics": None, "message": "No embedded lyrics found"}
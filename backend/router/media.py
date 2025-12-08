from fastapi import APIRouter, Depends, Response
from sqlmodel import Session, select
from database import get_session
from models import Album, Song
from mutagen import File as MutagenFile
from mutagen.id3 import ID3, APIC
from mutagen.flac import FLAC, Picture
from mutagen.mp4 import MP4
import os
import requests
import base64

router = APIRouter(tags=["Media"])

# 1x1 Transparent PNG pixel
DEFAULT_COVER = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=")

@router.get("/covers/{album_id}")
def get_album_cover(album_id: int, session: Session = Depends(get_session)):
    album = session.get(Album, album_id)
    if not album: 
        return Response(content=DEFAULT_COVER, media_type="image/png")
    
    # Check first 3 songs for art
    songs = session.exec(select(Song).where(Song.album_id == album_id).limit(3)).all()
    
    for song in songs:
        if not os.path.exists(song.path): continue
        try:
            audio = MutagenFile(song.path)
            if audio is None:
                continue
            
            image_data = None
            
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
                return Response(content=image_data, media_type="image/jpeg")
        except:
            continue

    # Return silent placeholder instead of 404 to fix console warnings
    return Response(content=DEFAULT_COVER, media_type="image/png")

@router.get("/lyrics/{song_id}")
def get_lyrics(song_id: int, session: Session = Depends(get_session)):
    song = session.get(Song, song_id)
    if not song: return Response(status_code=404)
    
    # 1. Check DB Cache
    if song.synced_lyrics:
        return {"plainLyrics": song.lyrics, "syncedLyrics": song.synced_lyrics}
    if song.lyrics:
        # If we only have plain lyrics, we can still return them, but maybe we want to try fetching synced?
        # For now, let's return what we have to be fast.
        pass # Fallthrough to see if we can get synced lyrics from API if only plain exists? 
             # Actually, if we have plain lyrics, the previous scanner logic would have put them in. 
             # If `song.lyrics` has timestamps, it would be in `synced_lyrics` too.
             # Let's trust the cache. If user wants to force refresh we'd need a separate endpoint.
             # Wait, if we only have plain, we might want to try to get synced from API.
             # But let's check API only if NO lyrics or if we explicitly want synced. 
             # To be safe and fast: If we have ANY lyrics, return them.
        return {"plainLyrics": song.lyrics, "syncedLyrics": None}

    # 2. Check File (Fallback for unscanned files or if DB empty)
    # This acts as a "just-in-time" scanner
    if os.path.exists(song.path):
        try:
            audio = MutagenFile(song.path, easy=True)
            if audio:
                # Mutagen 'lyrics' often maps to USLT
                raw_lyrics = None
                if 'lyrics' in audio:
                    raw_lyrics = audio['lyrics'][0]
                
                if raw_lyrics:
                    # Check for synced
                    is_synced = '[' in raw_lyrics and ']' in raw_lyrics
                    
                    # Update DB
                    song.lyrics = raw_lyrics
                    if is_synced:
                        song.synced_lyrics = raw_lyrics
                    
                    session.add(song)
                    session.commit()
                    
                    return {"plainLyrics": raw_lyrics, "syncedLyrics": raw_lyrics if is_synced else None}
        except Exception:
            pass

    # 3. Fetch from API
    # Logic: Try exact match -> Try Search -> Save -> Return
    params = {
        "track_name": song.title, 
        "artist_name": song.artist, 
        "album_name": song.album.title if song.album else "",
        "duration": song.duration
    }
    
    try:
        # Try finding exact match first (using duration helps precision)
        match = None
        
        # Method A: GET /get (Exact)
        try:
            resp = requests.get("https://lrclib.net/api/get", params=params, timeout=3)
            if resp.status_code == 200:
                match = resp.json()
        except: pass
        
        # Method B: Search if exact failed
        if not match:
            search_query = f"{song.title} {song.artist}"
            search_resp = requests.get("https://lrclib.net/api/search", params={"q": search_query}, timeout=3)
            if search_resp.status_code == 200:
                results = search_resp.json()
                if results and isinstance(results, list):
                    match = results[0] # Take best match
        
        if match:
            plain = match.get("plainLyrics")
            synced = match.get("syncedLyrics")
            
            # Cache to DB
            song.lyrics = plain
            song.synced_lyrics = synced
            session.add(song)
            session.commit()
            
            return {"plainLyrics": plain, "syncedLyrics": synced}
            
    except Exception as e:
        print(f"Lyrics fetch error: {e}")
        pass
    
    # 4. No lyrics found
    return {"plainLyrics": "No lyrics found."}
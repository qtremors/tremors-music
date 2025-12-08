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
    
    params = {"track_name": song.title, "artist_name": song.artist, "album_name": song.album.title if song.album else ""}
    try:
        resp = requests.get("https://lrclib.net/api/get", params=params, timeout=3)
        if resp.status_code == 200: return resp.json()
        
        search = requests.get("https://lrclib.net/api/search", params={"q": f"{song.title} {song.artist}"}, timeout=3)
        if search.status_code == 200 and search.json():
             return search.json()[0]
    except: pass
    
    return {"plainLyrics": "No lyrics found."}
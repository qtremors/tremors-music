from typing import List, Dict, Any
from fastapi import APIRouter, Depends, BackgroundTasks, HTTPException
from sqlmodel import Session, select, func, or_
from database import get_session
from models import Song, Album, LibraryPath
from scanner import scan_directory
import os
import urllib.parse

router = APIRouter(prefix="/library", tags=["Library"])

# --- PATH MANAGEMENT ---
@router.get("/paths", response_model=List[LibraryPath])
def get_paths(session: Session = Depends(get_session)):
    return session.exec(select(LibraryPath)).all()

@router.post("/paths")
def add_path(path_data: LibraryPath, session: Session = Depends(get_session)):
    if not os.path.exists(path_data.path):
        raise HTTPException(status_code=400, detail="Directory does not exist")
    existing = session.exec(select(LibraryPath).where(LibraryPath.path == path_data.path)).first()
    if existing: return existing
    session.add(path_data)
    session.commit()
    session.refresh(path_data)
    return path_data

@router.delete("/paths/{path_id}")
def remove_path(path_id: int, session: Session = Depends(get_session)):
    path_obj = session.get(LibraryPath, path_id)
    if not path_obj: raise HTTPException(status_code=404, detail="Path not found")
    session.delete(path_obj)
    session.commit()
    return {"message": "Path removed"}

@router.patch("/paths/{path_id}")
def update_library_path(path_id: int, new_path: str, session: Session = Depends(get_session)):
    """Update an existing library path."""
    path_obj = session.get(LibraryPath, path_id)
    if not path_obj:
        raise HTTPException(status_code=404, detail="Path not found")
    if not os.path.exists(new_path):
        raise HTTPException(status_code=400, detail="Directory does not exist")
    existing = session.exec(select(LibraryPath).where(LibraryPath.path == new_path)).first()
    if existing and existing.id != path_id:
        raise HTTPException(status_code=400, detail="Path already exists in library")
    path_obj.path = new_path
    session.commit()
    session.refresh(path_obj)
    return path_obj

# --- LIBRARY MANAGEMENT ---
@router.delete("/reset")
def reset_library(session: Session = Depends(get_session)):
    """Wipes Songs and Albums but KEEPS Paths."""
    session.exec(delete(Song))
    session.exec(delete(Album))
    session.commit()
    return {"message": "Library reset (Paths saved)"}

@router.post("/scan")
def scan_library(background_tasks: BackgroundTasks, session: Session = Depends(get_session)):
    paths = session.exec(select(LibraryPath)).all()
    if not paths: raise HTTPException(status_code=400, detail="No paths configured.")
    
    # Import here to avoid circular dependency
    from scanner_progress import scanner_progress
    
    # Reset progress before starting
    scanner_progress.reset()
    
    for p in paths:
        background_tasks.add_task(scan_directory, os.path.normpath(p.path))
    return {"message": "Scanning started"}

@router.get("/scan/status")
def get_scan_status():
    """Get real-time scanner progress"""
    from scanner_progress import scanner_progress
    return scanner_progress.to_dict()

@router.post("/scan/stop")
def stop_scan():
    """Stop the currently running scan."""
    from scanner_progress import scanner_progress
    if not scanner_progress.is_scanning:
        raise HTTPException(status_code=400, detail="No scan is currently running")
    scanner_progress.finish()
    return {"message": "Scan stopped"}

# --- ARTISTS ---
@router.get("/artists")
def get_artists(session: Session = Depends(get_session)):
    albums = session.exec(select(Album)).all()
    artist_map = {}
    for album in albums:
        if album.artist not in artist_map:
            artist_map[album.artist] = {
                "name": album.artist,
                "album_count": 0,
                "cover_example": album.id 
            }
        artist_map[album.artist]["album_count"] += 1
    return list(artist_map.values())

@router.get("/artists/{artist_name}/work")
def get_artist_work(artist_name: str, session: Session = Depends(get_session)):
    decoded_name = urllib.parse.unquote(artist_name).lower()
    
    # Join with Album to check Album Artist too
    songs = session.exec(
        select(Song)
        .join(Album, isouter=True)
        .where(
            or_(
                func.lower(Song.artist) == decoded_name,
                func.lower(Album.artist) == decoded_name
            )
        )
        .order_by(Song.album_id, Song.track_number)
    ).all()
    
    work_map = {}
    
    for song in songs:
        alb_id = song.album_id or -1
        if alb_id not in work_map:
            album_data = None
            if alb_id != -1:
                album_data = session.get(Album, alb_id)
            
            work_map[alb_id] = {
                "album": album_data,
                "songs": []
            }
        work_map[alb_id]["songs"].append(song)
        
    return list(work_map.values())

# --- SONGS & SORTING ---
@router.get("/songs", response_model=List[Song])
def get_songs(
    offset: int = 0, 
    limit: int = 5000, 
    sort_by: str = "title", 
    order: str = "asc", 
    session: Session = Depends(get_session)
):
    query = select(Song)
    
    # Case-insensitive sorting logic
    if sort_by == "title":
        sort_col = func.lower(Song.title)
    elif sort_by == "artist":
        sort_col = func.lower(Song.artist)
    elif sort_by == "album":
        query = query.join(Album, isouter=True)
        sort_col = func.lower(Album.title)
    elif sort_by == "duration":
        sort_col = Song.duration
    elif sort_by == "file_size":
        sort_col = Song.file_size
    elif sort_by == "id":
        sort_col = Song.id
    else:
        sort_col = func.lower(Song.title)

    if order == "desc":
        query = query.order_by(sort_col.desc())
    else:
        query = query.order_by(sort_col.asc())

    return session.exec(query.offset(offset).limit(limit)).all()

# --- SEARCH ---
@router.get("/search", response_model=Dict[str, Any])
def search_library(q: str = "", limit: int = 50, session: Session = Depends(get_session)):
    """
    Instant search across songs, albums, and artists.
    Returns results grouped by type.
    """
    if not q:
        return {"songs": [], "albums": [], "artists": []}
    
    search_term = f"%{q}%"
    
    # Search songs - join with Album to search album titles
    songs = session.exec(
        select(Song).join(Album, isouter=True).where(
            or_(
                Song.title.ilike(search_term),
                Song.artist.ilike(search_term),
                Album.title.ilike(search_term)
            )
        ).limit(limit)
    ).all()
    
    # Search albums
    albums = session.exec(
        select(Album).where(
            or_(
                Album.title.ilike(search_term),
                Album.artist.ilike(search_term)
            )
        ).limit(20)
    ).all()
    
    # Get unique artists from songs
    artists_query = session.exec(
        select(Song.artist).where(
            Song.artist.ilike(search_term)
        ).distinct().limit(20)
    ).all()
    
    artists = [{"name": artist} for artist in artists_query if artist]
    
    return {
        "songs": songs,
        "albums": albums,
        "artists": artists
    }

# --- ALBUMS ---
@router.get("/albums", response_model=List[Album])
def get_albums(offset: int = 0, limit: int = 50, session: Session = Depends(get_session)):
    return session.exec(select(Album).offset(offset).limit(limit)).all()

@router.get("/albums/{album_id}")
def get_album_details(album_id: int, session: Session = Depends(get_session)):
    return session.get(Album, album_id)

@router.get("/albums/{album_id}/songs", response_model=List[Song])
def get_album_songs(album_id: int, session: Session = Depends(get_session)):
    return session.exec(select(Song).where(Song.album_id == album_id).order_by(Song.track_number, Song.title)).all()
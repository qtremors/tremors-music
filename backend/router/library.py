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
    for p in paths:
        background_tasks.add_task(scan_directory, os.path.normpath(p.path))
    return {"message": "Scanning started"}

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
@router.get("/search", response_model=List[Song])
def search_songs(q: str, limit: int = 50, session: Session = Depends(get_session)):
    if not q: return []
    query = select(Song).join(Album, isouter=True).where(
        or_(
            Song.title.ilike(f"%{q}%"),
            Song.artist.ilike(f"%{q}%"),
            Album.title.ilike(f"%{q}%")
        )
    ).limit(limit)
    return session.exec(query).all()

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
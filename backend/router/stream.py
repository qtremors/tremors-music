from fastapi import APIRouter, Depends, HTTPException, Header
from fastapi.responses import StreamingResponse
from sqlmodel import Session
from database import get_session
from models import Song
import os
import mimetypes
from streamer import range_generator

router = APIRouter(prefix="/stream", tags=["Stream"])

@router.get("/{song_id}")
def stream_music(
    song_id: int,
    range: str = Header(None),
    session: Session = Depends(get_session)
):
    # 1. Find the song in DB
    song = session.get(Song, song_id)
    if not song:
        raise HTTPException(status_code=404, detail="Song not found")
    
    # 2. Verify file exists on disk
    if not os.path.exists(song.path):
        raise HTTPException(status_code=404, detail="File not found on disk")

    file_size = os.path.getsize(song.path)
    
    # 3. Handle "Range" Header (Seeking)
    start = 0
    end = file_size - 1
    
    if range:
        # Example header: "bytes=0-" or "bytes=1024-2048"
        try:
            start_str, end_str = range.replace("bytes=", "").split("-")
            start = int(start_str)
            if end_str:
                end = int(end_str)
        except ValueError:
            pass # Fallback to full file if header is malformed

    # Ensure valid range
    if start >= file_size:
        # If browser asks for a byte past the end, reset
        start = 0
        end = file_size - 1

    # 4. Calculate Content-Length for this specific chunk
    chunk_length = end - start + 1
    
    # 5. Guess MIME type (audio/mpeg, audio/flac, etc.)
    mime_type, _ = mimetypes.guess_type(song.path)
    if not mime_type:
        mime_type = "audio/mpeg" # Default fallback

    # 6. Return the 206 Partial Content response
    headers = {
        "Content-Range": f"bytes {start}-{end}/{file_size}",
        "Accept-Ranges": "bytes",
        "Content-Length": str(chunk_length),
        "Content-Type": mime_type,
    }
    
    return StreamingResponse(
        range_generator(song.path, start, end),
        status_code=206,
        headers=headers
    )
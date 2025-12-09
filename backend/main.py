import os
import sys
import logging
from logging.handlers import RotatingFileHandler
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from database import create_db_and_tables
from router import library, stream, media, playlists

# --- Logging Setup ---
def setup_logging():
    """Configure file-based logging for production."""
    # Get the directory where the executable is located
    if getattr(sys, 'frozen', False):
        # Running as compiled executable
        app_dir = os.path.dirname(sys.executable)
    else:
        # Running as script
        app_dir = os.path.dirname(os.path.abspath(__file__))
    
    log_dir = os.path.join(app_dir, 'logs')
    os.makedirs(log_dir, exist_ok=True)
    log_file = os.path.join(log_dir, 'tremorsmusic.log')
    
    # Configure rotating file handler (5MB max, keep 5 backups)
    handler = RotatingFileHandler(
        log_file,
        maxBytes=5*1024*1024,  # 5MB
        backupCount=5,
        encoding='utf-8'
    )
    handler.setFormatter(logging.Formatter(
        '%(asctime)s - %(levelname)s - %(name)s - %(message)s'
    ))
    
    # Configure root logger
    logging.basicConfig(
        level=logging.INFO,
        handlers=[handler]
    )
    
    # Also configure uvicorn loggers
    for logger_name in ['uvicorn', 'uvicorn.access', 'uvicorn.error']:
        logger = logging.getLogger(logger_name)
        logger.handlers = [handler]
    
    logging.info("Tremors Music Backend starting...")
    logging.info(f"App directory: {app_dir}")
    logging.info(f"Log file: {log_file}")

# Initialize logging
setup_logging()

@asynccontextmanager
async def lifespan(app: FastAPI):
    logging.info("Initializing database...")
    create_db_and_tables()
    logging.info("Database ready. Backend is now accepting connections.")
    yield
    logging.info("Backend shutting down...")

app = FastAPI(lifespan=lifespan)

# CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Register Routers
app.include_router(library.router)
app.include_router(stream.router)
app.include_router(media.router)
app.include_router(playlists.router)


@app.get("/")
def read_root():
    return {"message": "Tremors Music Backend is Ready 🎵", "version": "1.5.0"}
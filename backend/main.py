from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from database import create_db_and_tables
from router import library, stream, media, playlists

@asynccontextmanager
async def lifespan(app: FastAPI):
    create_db_and_tables()
    yield

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
    return {"message": "Tremors Music Backend is Ready 🎵"}
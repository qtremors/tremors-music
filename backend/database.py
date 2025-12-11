import os
import sys
from sqlmodel import SQLModel, create_engine, Session

# Get the directory where the executable/script is located
def get_app_dir():
    """Get the application directory (where the executable is located)."""
    if getattr(sys, 'frozen', False):
        # Running as compiled executable
        return os.path.dirname(sys.executable)
    else:
        # Running as script (development)
        return os.path.dirname(os.path.abspath(__file__))

# 1. Define the database file path (in app directory)
app_dir = get_app_dir()
sqlite_file_name = os.path.join(app_dir, "music.db")
sqlite_url = f"sqlite:///{sqlite_file_name}"

# 2. Create the engine
engine = create_engine(sqlite_url, connect_args={"check_same_thread": False})

# 3. Helper to create tables
def create_db_and_tables():
    SQLModel.metadata.create_all(engine)

# 4. Dependency Injection helper
def get_session():
    with Session(engine) as session:
        yield session
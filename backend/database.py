from sqlmodel import SQLModel, create_engine, Session

# 1. Define the database file name
sqlite_file_name = "music.db"
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
"""
Entry point for PyInstaller build.
This script starts the FastAPI backend server.
"""
import uvicorn
import os
import multiprocessing
import sys

if __name__ == "__main__":
    multiprocessing.freeze_support()
    
    # Allow port configuration via environment variable (useful for dev/custom setups)
    port = int(os.environ.get("TREMORS_PORT", 8000))
    
    # Determine app directory for logging context (backend logic handles this, but good to know)
    # We bind strictly to localhost for security
    uvicorn.run("main:app", host="127.0.0.1", port=port, reload=False, workers=1)

# Scanner progress tracking
from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional, Dict, Any, List
import threading

@dataclass
class ErrorDetail:
    """Details about a scan error"""
    file_path: str
    error_message: str
    timestamp: str

@dataclass
class ScanProgress:
    is_scanning: bool = False
    files_processed: int = 0
    songs_added: int = 0
    errors: int = 0
    current_file: str = ""
    start_time: Optional[float] = None
    
    # Track error details
    error_details: List[ErrorDetail] = field(default_factory=list)
    
    # Persistent last scan result
    last_scan_result: Optional[Dict[str, Any]] = None
    
    _lock: threading.Lock = field(default_factory=threading.Lock, repr=False)
    
    def reset(self):
        with self._lock:
            self.is_scanning = True
            self.files_processed = 0
            self.songs_added = 0
            self.errors = 0
            self.current_file = ""
            self.start_time = datetime.now().timestamp()
            self.error_details = []
    
    def update(self, files: int = 0, songs: int = 0, errors: int = 0, current: str = "", error_file: str = "", error_msg: str = ""):
        with self._lock:
            self.files_processed += files
            self.songs_added += songs
            self.errors += errors
            if current:
                self.current_file = current
            if error_file and error_msg:
                # Limit to last 50 errors to avoid memory issues
                if len(self.error_details) < 50:
                    self.error_details.append(ErrorDetail(
                        file_path=error_file,
                        error_message=error_msg,
                        timestamp=datetime.now().isoformat()
                    ))
    
    def finish(self):
        with self._lock:
            # Save result before finishing
            if self.start_time:
                duration = datetime.now().timestamp() - self.start_time
                self.last_scan_result = {
                    "files_processed": self.files_processed,
                    "songs_added": self.songs_added,
                    "errors": self.errors,
                    "duration": duration,
                    "completed_at": datetime.now().isoformat(),
                    "error_details": [
                        {
                            "file_path": e.file_path,
                            "error_message": e.error_message,
                            "timestamp": e.timestamp
                        }
                        for e in self.error_details
                    ]
                }
            
            self.is_scanning = False
            self.current_file = ""
    
    def to_dict(self):
        with self._lock:
            return {
                "is_scanning": self.is_scanning,
                "files_processed": self.files_processed,
                "songs_added": self.songs_added,
                "errors": self.errors,
                "current_file": self.current_file,
                "start_time": self.start_time,
                "error_details": [
                    {
                        "file_path": e.file_path,
                        "error_message": e.error_message,
                        "timestamp": e.timestamp
                    }
                    for e in self.error_details
                ],
                "last_scan_result": self.last_scan_result
            }

# Global scanner state
scanner_progress = ScanProgress()

def range_generator(file_path: str, start: int, end: int, chunk_size: int = 1024 * 1024):
    """
    Yields chunks of data from a file between start and end bytes.
    """
    with open(file_path, "rb") as f:
        f.seek(start)
        bytes_to_read = end - start + 1
        
        while bytes_to_read > 0:
            chunk = f.read(min(chunk_size, bytes_to_read))
            if not chunk:
                break
            yield chunk
            bytes_to_read -= len(chunk)
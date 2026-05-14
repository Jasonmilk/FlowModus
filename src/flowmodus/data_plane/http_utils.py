"""HTTP status code classification for error handling."""
from typing import Optional


def classify_http_error(status_code: int, exception: Optional[Exception] = None) -> str:
    """Classify HTTP errors into 'success', 'retryable', or 'terminal'."""
    if 200 <= status_code < 300:
        return "success"
    if status_code in (429, 500, 502, 503, 504):
        return "retryable"
    # Connection errors are retryable
    if exception is not None and "Connect" in type(exception).__name__:
        return "retryable"
    if status_code in (401, 403, 400):
        return "terminal"
    # Unknown errors are retryable once
    return "retryable"

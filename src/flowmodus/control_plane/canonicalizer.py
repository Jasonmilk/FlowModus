import json
from typing import Any


def canonicalize_json(data: Any) -> bytes:
    """
    Canonicalize JSON data to ensure deterministic signing.
    Sorts keys recursively, no extra whitespace.
    Pure function.
    
    Args:
        data: JSON data
    
    Returns:
        Canonicalized bytes
    """
    def _sort_dict(d: dict) -> dict:
        return {k: _canonicalize(v) for k, v in sorted(d.items())}
    
    def _canonicalize(v: Any) -> Any:
        if isinstance(v, dict):
            return _sort_dict(v)
        elif isinstance(v, list):
            return [_canonicalize(item) for item in v]
        else:
            return v
    
    canonical_data = _canonicalize(data)
    return json.dumps(canonical_data, separators=(',', ':'), ensure_ascii=False).encode('utf-8')

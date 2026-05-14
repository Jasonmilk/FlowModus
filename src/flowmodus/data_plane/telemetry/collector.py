import time
from typing import Optional, Dict, Any
import asyncio

from flowmodus.schemas.metrics_pb2 import TelemetrySample
from .deviation import calculate_deviation
from ..proxy import classify_http_error


class TTFTTimedStream:
    """
    Wrapper for response stream to measure Time To First Token (TTFT).
    Zero-burden, no polling, only measures on first chunk.
    """
    
    def __init__(self, response_stream, start_time: float):
        self._stream = response_stream
        self._start_time = start_time
        self._first_chunk = True
        self.ttft_ms: Optional[float] = None
    
    async def __aiter__(self):
        async for chunk in self._stream:
            if self._first_chunk:
                self.ttft_ms = (time.monotonic() - self._start_time) * 1000
                self._first_chunk = False
            yield chunk


class TelemetryCollector:
    """
    Passive telemetry collector.
    Collects metrics from real business requests, no active probing.
    """
    
    def __init__(self):
        self._db_connection = None
        self._init_db()
    
    def _init_db(self) -> None:
        """Initialize local SQLite database."""
        # TODO: Implement SQLite schema creation
        pass
    
    def collect_from_response(
        self,
        supplier_id: str,
        model_id: str,
        status_code: int,
        ttft_ms: float,
        total_duration_ms: float,
        response_headers: Dict[str, str],
        usage: Dict[str, int],
        error: Optional[Exception] = None,
    ) -> TelemetrySample:
        """
        Collect telemetry sample from a completed response.
        
        Args:
            supplier_id: Supplier ID
            model_id: Model ID
            status_code: HTTP status code
            ttft_ms: Time to first token in ms
            total_duration_ms: Total request duration in ms
            response_headers: Response headers
            usage: Usage stats from response
            error: Exception if any
        
        Returns:
            Telemetry sample
        """
        health_state = classify_http_error(status_code, error)
        
        sample = TelemetrySample(
            supplier_id=supplier_id,
            model_id=model_id,
            sample_time_unix=int(time.time()),
            ttft_ms=ttft_ms,
            total_duration_ms=total_duration_ms,
            kv_cache_hit=self._check_cache_hit(response_headers),
            billed_tokens=usage.get("total_tokens", 0),
            status_code=status_code,
            health_state=health_state,
        )
        
        # Store sample locally
        self._store_sample(sample)
        
        return sample
    
    def _check_cache_hit(self, response_headers: Dict[str, str]) -> bool:
        """Check if KV cache hit from response headers."""
        # TODO: Implement cache hit detection
        return False
    
    def _store_sample(self, sample: TelemetrySample) -> None:
        """Store sample in local database."""
        # TODO: Implement SQLite insert
        pass
    
    def flush(self) -> None:
        """Flush any pending telemetry data to disk."""
        # TODO: Implement flush
        pass

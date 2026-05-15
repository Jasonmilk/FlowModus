import time
import sqlite3
import os
from dataclasses import dataclass, field
from typing import Optional, Dict, Any, List

from flowmodus.schemas.metrics_pb2 import TelemetrySample
from .deviation import calculate_deviation
from flowmodus.data_plane.http_utils import classify_http_error


@dataclass
class DeviationSnapshot:
    """Immutable snapshot of claim deviations, read by the pipeline."""
    deviations: Dict[str, Dict[str, float]] = field(default_factory=dict)

    def get(self, key: str, default: Any = None) -> Any:
        """Dict-like access to supplier deviation data."""
        return self.deviations.get(key, default)

    def get_deviation(self, supplier_id: str, metric_name: str) -> Optional[float]:
        return self.deviations.get(supplier_id, {}).get(metric_name)


class TTFTTimedStream:
    """Wrapper for response stream to measure Time To First Token (TTFT)."""

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
    """Passive telemetry collector. Writes to SQLite, updates in-memory snapshot."""

    DB_PATH = "~/.flowmodus/metrics.db"

    def __init__(self, snapshot: Optional[DeviationSnapshot] = None):
        self._snapshot = snapshot or DeviationSnapshot()
        self._db_connection = None
        self._init_db()

    def _init_db(self) -> None:
        db_path = os.path.expanduser(self.DB_PATH)
        os.makedirs(os.path.dirname(db_path), exist_ok=True)
        self._db_connection = sqlite3.connect(db_path, check_same_thread=False)
        self._db_connection.execute("""
            CREATE TABLE IF NOT EXISTS telemetry_samples (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                supplier_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                sample_time_unix INTEGER NOT NULL,
                ttft_ms REAL NOT NULL,
                total_duration_ms REAL NOT NULL,
                kv_cache_hit INTEGER NOT NULL,
                billed_tokens INTEGER NOT NULL,
                status_code INTEGER NOT NULL,
                health_state TEXT NOT NULL
            );
        """)
        self._db_connection.execute("""
            CREATE TABLE IF NOT EXISTS claim_deviations (
                supplier_id TEXT NOT NULL,
                metric_name TEXT NOT NULL,
                claimed_value REAL NOT NULL,
                actual_value REAL NOT NULL,
                deviation_percent REAL NOT NULL,
                calculated_at_unix INTEGER NOT NULL,
                PRIMARY KEY (supplier_id, metric_name, calculated_at_unix)
            );
        """)
        self._db_connection.commit()

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
        self._store_sample(sample)
        new_snapshot = self._build_snapshot()
        self._snapshot = new_snapshot
        return sample

    def _check_cache_hit(self, response_headers: Dict[str, str]) -> bool:
        if response_headers.get("x-cache") == "HIT":
            return True
        if response_headers.get("anthropic-cache") == "hit":
            return True
        return False

    def _store_sample(self, sample: TelemetrySample) -> None:
        if not self._db_connection:
            return
        self._db_connection.execute(
            "INSERT INTO telemetry_samples (supplier_id, model_id, sample_time_unix, ttft_ms, total_duration_ms, kv_cache_hit, billed_tokens, status_code, health_state) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (sample.supplier_id, sample.model_id, sample.sample_time_unix, sample.ttft_ms, sample.total_duration_ms, int(sample.kv_cache_hit), sample.billed_tokens, sample.status_code, sample.health_state),
        )
        self._db_connection.commit()

    def _build_snapshot(self) -> DeviationSnapshot:
        raw_deviations: Dict[str, Dict[str, float]] = {}
        try:
            cursor = self._db_connection.execute(
                "SELECT supplier_id, metric_name, deviation_percent FROM claim_deviations ORDER BY calculated_at_unix DESC LIMIT 1000"
            )
            for row in cursor.fetchall():
                sid, metric, val = row
                if sid not in raw_deviations:
                    raw_deviations[sid] = {}
                raw_deviations[sid][metric] = val
        except Exception:
            pass
        return DeviationSnapshot(deviations=raw_deviations)

    @property
    def snapshot(self) -> DeviationSnapshot:
        return self._snapshot

    def flush(self) -> None:
        if self._db_connection:
            self._db_connection.commit()

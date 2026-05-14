from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class TelemetrySample(_message.Message):
    __slots__ = ("supplier_id", "model_id", "sample_time_unix", "ttft_ms", "total_duration_ms", "kv_cache_hit", "billed_tokens", "status_code", "health_state")
    SUPPLIER_ID_FIELD_NUMBER: _ClassVar[int]
    MODEL_ID_FIELD_NUMBER: _ClassVar[int]
    SAMPLE_TIME_UNIX_FIELD_NUMBER: _ClassVar[int]
    TTFT_MS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_DURATION_MS_FIELD_NUMBER: _ClassVar[int]
    KV_CACHE_HIT_FIELD_NUMBER: _ClassVar[int]
    BILLED_TOKENS_FIELD_NUMBER: _ClassVar[int]
    STATUS_CODE_FIELD_NUMBER: _ClassVar[int]
    HEALTH_STATE_FIELD_NUMBER: _ClassVar[int]
    supplier_id: str
    model_id: str
    sample_time_unix: int
    ttft_ms: float
    total_duration_ms: float
    kv_cache_hit: bool
    billed_tokens: int
    status_code: int
    health_state: str
    def __init__(self, supplier_id: _Optional[str] = ..., model_id: _Optional[str] = ..., sample_time_unix: _Optional[int] = ..., ttft_ms: _Optional[float] = ..., total_duration_ms: _Optional[float] = ..., kv_cache_hit: bool = ..., billed_tokens: _Optional[int] = ..., status_code: _Optional[int] = ..., health_state: _Optional[str] = ...) -> None: ...

class ClaimDeviation(_message.Message):
    __slots__ = ("supplier_id", "metric_name", "claimed_value", "actual_value", "deviation_percent", "calculated_at_unix")
    SUPPLIER_ID_FIELD_NUMBER: _ClassVar[int]
    METRIC_NAME_FIELD_NUMBER: _ClassVar[int]
    CLAIMED_VALUE_FIELD_NUMBER: _ClassVar[int]
    ACTUAL_VALUE_FIELD_NUMBER: _ClassVar[int]
    DEVIATION_PERCENT_FIELD_NUMBER: _ClassVar[int]
    CALCULATED_AT_UNIX_FIELD_NUMBER: _ClassVar[int]
    supplier_id: str
    metric_name: str
    claimed_value: float
    actual_value: float
    deviation_percent: float
    calculated_at_unix: int
    def __init__(self, supplier_id: _Optional[str] = ..., metric_name: _Optional[str] = ..., claimed_value: _Optional[float] = ..., actual_value: _Optional[float] = ..., deviation_percent: _Optional[float] = ..., calculated_at_unix: _Optional[int] = ...) -> None: ...

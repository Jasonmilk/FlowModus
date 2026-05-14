from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RawRequest(_message.Message):
    __slots__ = ("prompt", "agent_role", "cognitive_mode", "max_output_tokens", "extra_headers")
    class ExtraHeadersEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    AGENT_ROLE_FIELD_NUMBER: _ClassVar[int]
    COGNITIVE_MODE_FIELD_NUMBER: _ClassVar[int]
    MAX_OUTPUT_TOKENS_FIELD_NUMBER: _ClassVar[int]
    EXTRA_HEADERS_FIELD_NUMBER: _ClassVar[int]
    prompt: str
    agent_role: str
    cognitive_mode: str
    max_output_tokens: int
    extra_headers: _containers.ScalarMap[str, str]
    def __init__(self, prompt: _Optional[str] = ..., agent_role: _Optional[str] = ..., cognitive_mode: _Optional[str] = ..., max_output_tokens: _Optional[int] = ..., extra_headers: _Optional[_Mapping[str, str]] = ...) -> None: ...

class NormalizedRequest(_message.Message):
    __slots__ = ("prompt_hash", "estimated_input_tokens", "max_output_tokens", "agent_role", "cognitive_mode", "ste_input", "ste_output_estimated", "cache_breakpoints")
    PROMPT_HASH_FIELD_NUMBER: _ClassVar[int]
    ESTIMATED_INPUT_TOKENS_FIELD_NUMBER: _ClassVar[int]
    MAX_OUTPUT_TOKENS_FIELD_NUMBER: _ClassVar[int]
    AGENT_ROLE_FIELD_NUMBER: _ClassVar[int]
    COGNITIVE_MODE_FIELD_NUMBER: _ClassVar[int]
    STE_INPUT_FIELD_NUMBER: _ClassVar[int]
    STE_OUTPUT_ESTIMATED_FIELD_NUMBER: _ClassVar[int]
    CACHE_BREAKPOINTS_FIELD_NUMBER: _ClassVar[int]
    prompt_hash: str
    estimated_input_tokens: int
    max_output_tokens: int
    agent_role: str
    cognitive_mode: str
    ste_input: float
    ste_output_estimated: float
    cache_breakpoints: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, prompt_hash: _Optional[str] = ..., estimated_input_tokens: _Optional[int] = ..., max_output_tokens: _Optional[int] = ..., agent_role: _Optional[str] = ..., cognitive_mode: _Optional[str] = ..., ste_input: _Optional[float] = ..., ste_output_estimated: _Optional[float] = ..., cache_breakpoints: _Optional[_Iterable[int]] = ...) -> None: ...

class CostEstimate(_message.Message):
    __slots__ = ("supplier_id", "model_id", "estimated_cost_usd", "ste_total", "kv_cache_applicable", "kv_cache_savings_estimate")
    SUPPLIER_ID_FIELD_NUMBER: _ClassVar[int]
    MODEL_ID_FIELD_NUMBER: _ClassVar[int]
    ESTIMATED_COST_USD_FIELD_NUMBER: _ClassVar[int]
    STE_TOTAL_FIELD_NUMBER: _ClassVar[int]
    KV_CACHE_APPLICABLE_FIELD_NUMBER: _ClassVar[int]
    KV_CACHE_SAVINGS_ESTIMATE_FIELD_NUMBER: _ClassVar[int]
    supplier_id: str
    model_id: str
    estimated_cost_usd: float
    ste_total: float
    kv_cache_applicable: bool
    kv_cache_savings_estimate: float
    def __init__(self, supplier_id: _Optional[str] = ..., model_id: _Optional[str] = ..., estimated_cost_usd: _Optional[float] = ..., ste_total: _Optional[float] = ..., kv_cache_applicable: bool = ..., kv_cache_savings_estimate: _Optional[float] = ...) -> None: ...

class UserConstraints(_message.Message):
    __slots__ = ("max_cost_per_request_usd", "require_regions", "require_modalities", "require_verified_supplier", "max_claim_deviation_tolerance")
    MAX_COST_PER_REQUEST_USD_FIELD_NUMBER: _ClassVar[int]
    REQUIRE_REGIONS_FIELD_NUMBER: _ClassVar[int]
    REQUIRE_MODALITIES_FIELD_NUMBER: _ClassVar[int]
    REQUIRE_VERIFIED_SUPPLIER_FIELD_NUMBER: _ClassVar[int]
    MAX_CLAIM_DEVIATION_TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    max_cost_per_request_usd: float
    require_regions: _containers.RepeatedScalarFieldContainer[str]
    require_modalities: _containers.RepeatedScalarFieldContainer[str]
    require_verified_supplier: bool
    max_claim_deviation_tolerance: float
    def __init__(self, max_cost_per_request_usd: _Optional[float] = ..., require_regions: _Optional[_Iterable[str]] = ..., require_modalities: _Optional[_Iterable[str]] = ..., require_verified_supplier: bool = ..., max_claim_deviation_tolerance: _Optional[float] = ...) -> None: ...

class EligibleSupplier(_message.Message):
    __slots__ = ("supplier_id", "model_id", "endpoint_url", "score", "claim_deviation", "cost")
    SUPPLIER_ID_FIELD_NUMBER: _ClassVar[int]
    MODEL_ID_FIELD_NUMBER: _ClassVar[int]
    ENDPOINT_URL_FIELD_NUMBER: _ClassVar[int]
    SCORE_FIELD_NUMBER: _ClassVar[int]
    CLAIM_DEVIATION_FIELD_NUMBER: _ClassVar[int]
    COST_FIELD_NUMBER: _ClassVar[int]
    supplier_id: str
    model_id: str
    endpoint_url: str
    score: float
    claim_deviation: float
    cost: CostEstimate
    def __init__(self, supplier_id: _Optional[str] = ..., model_id: _Optional[str] = ..., endpoint_url: _Optional[str] = ..., score: _Optional[float] = ..., claim_deviation: _Optional[float] = ..., cost: _Optional[_Union[CostEstimate, _Mapping]] = ...) -> None: ...

class RoutingDecision(_message.Message):
    __slots__ = ("supplier_id", "model_id", "endpoint_url", "estimated_cost_usd", "kv_cache_parameter", "kv_cache_ttl", "request_id")
    SUPPLIER_ID_FIELD_NUMBER: _ClassVar[int]
    MODEL_ID_FIELD_NUMBER: _ClassVar[int]
    ENDPOINT_URL_FIELD_NUMBER: _ClassVar[int]
    ESTIMATED_COST_USD_FIELD_NUMBER: _ClassVar[int]
    KV_CACHE_PARAMETER_FIELD_NUMBER: _ClassVar[int]
    KV_CACHE_TTL_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    supplier_id: str
    model_id: str
    endpoint_url: str
    estimated_cost_usd: float
    kv_cache_parameter: str
    kv_cache_ttl: int
    request_id: str
    def __init__(self, supplier_id: _Optional[str] = ..., model_id: _Optional[str] = ..., endpoint_url: _Optional[str] = ..., estimated_cost_usd: _Optional[float] = ..., kv_cache_parameter: _Optional[str] = ..., kv_cache_ttl: _Optional[int] = ..., request_id: _Optional[str] = ...) -> None: ...

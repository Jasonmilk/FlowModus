from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SupplierDeclaration(_message.Message):
    __slots__ = ("supplier_id", "supplier_name", "verified", "updated_at_unix", "models", "endpoints", "compliance", "rate_limits")
    SUPPLIER_ID_FIELD_NUMBER: _ClassVar[int]
    SUPPLIER_NAME_FIELD_NUMBER: _ClassVar[int]
    VERIFIED_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_UNIX_FIELD_NUMBER: _ClassVar[int]
    MODELS_FIELD_NUMBER: _ClassVar[int]
    ENDPOINTS_FIELD_NUMBER: _ClassVar[int]
    COMPLIANCE_FIELD_NUMBER: _ClassVar[int]
    RATE_LIMITS_FIELD_NUMBER: _ClassVar[int]
    supplier_id: str
    supplier_name: str
    verified: bool
    updated_at_unix: int
    models: _containers.RepeatedCompositeFieldContainer[ModelDeclaration]
    endpoints: _containers.RepeatedCompositeFieldContainer[EndpointDeclaration]
    compliance: ComplianceDeclaration
    rate_limits: RateLimits
    def __init__(self, supplier_id: _Optional[str] = ..., supplier_name: _Optional[str] = ..., verified: bool = ..., updated_at_unix: _Optional[int] = ..., models: _Optional[_Iterable[_Union[ModelDeclaration, _Mapping]]] = ..., endpoints: _Optional[_Iterable[_Union[EndpointDeclaration, _Mapping]]] = ..., compliance: _Optional[_Union[ComplianceDeclaration, _Mapping]] = ..., rate_limits: _Optional[_Union[RateLimits, _Mapping]] = ...) -> None: ...

class ModelDeclaration(_message.Message):
    __slots__ = ("model_id", "display_name", "lang", "semantic_tags", "agent_roles", "billing", "kv_cache", "capabilities", "tokenizer_compression_ratio", "capability_tags")
    MODEL_ID_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_NAME_FIELD_NUMBER: _ClassVar[int]
    LANG_FIELD_NUMBER: _ClassVar[int]
    SEMANTIC_TAGS_FIELD_NUMBER: _ClassVar[int]
    AGENT_ROLES_FIELD_NUMBER: _ClassVar[int]
    BILLING_FIELD_NUMBER: _ClassVar[int]
    KV_CACHE_FIELD_NUMBER: _ClassVar[int]
    CAPABILITIES_FIELD_NUMBER: _ClassVar[int]
    TOKENIZER_COMPRESSION_RATIO_FIELD_NUMBER: _ClassVar[int]
    CAPABILITY_TAGS_FIELD_NUMBER: _ClassVar[int]
    model_id: str
    display_name: str
    lang: str
    semantic_tags: _containers.RepeatedScalarFieldContainer[str]
    agent_roles: _containers.RepeatedScalarFieldContainer[str]
    billing: BillingDeclaration
    kv_cache: KvCacheDeclaration
    capabilities: CapabilitiesDeclaration
    tokenizer_compression_ratio: float
    capability_tags: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, model_id: _Optional[str] = ..., display_name: _Optional[str] = ..., lang: _Optional[str] = ..., semantic_tags: _Optional[_Iterable[str]] = ..., agent_roles: _Optional[_Iterable[str]] = ..., billing: _Optional[_Union[BillingDeclaration, _Mapping]] = ..., kv_cache: _Optional[_Union[KvCacheDeclaration, _Mapping]] = ..., capabilities: _Optional[_Union[CapabilitiesDeclaration, _Mapping]] = ..., tokenizer_compression_ratio: _Optional[float] = ..., capability_tags: _Optional[_Iterable[str]] = ...) -> None: ...

class BillingDeclaration(_message.Message):
    __slots__ = ("currency", "token_input", "token_output", "compute_ms", "audio_sec", "video_frame", "free_quota_daily")
    CURRENCY_FIELD_NUMBER: _ClassVar[int]
    TOKEN_INPUT_FIELD_NUMBER: _ClassVar[int]
    TOKEN_OUTPUT_FIELD_NUMBER: _ClassVar[int]
    COMPUTE_MS_FIELD_NUMBER: _ClassVar[int]
    AUDIO_SEC_FIELD_NUMBER: _ClassVar[int]
    VIDEO_FRAME_FIELD_NUMBER: _ClassVar[int]
    FREE_QUOTA_DAILY_FIELD_NUMBER: _ClassVar[int]
    currency: str
    token_input: float
    token_output: float
    compute_ms: float
    audio_sec: float
    video_frame: float
    free_quota_daily: int
    def __init__(self, currency: _Optional[str] = ..., token_input: _Optional[float] = ..., token_output: _Optional[float] = ..., compute_ms: _Optional[float] = ..., audio_sec: _Optional[float] = ..., video_frame: _Optional[float] = ..., free_quota_daily: _Optional[int] = ...) -> None: ...

class KvCacheDeclaration(_message.Message):
    __slots__ = ("supported", "ttl_seconds", "control_parameter", "breakpoint_marker", "standard_compliance")
    SUPPORTED_FIELD_NUMBER: _ClassVar[int]
    TTL_SECONDS_FIELD_NUMBER: _ClassVar[int]
    CONTROL_PARAMETER_FIELD_NUMBER: _ClassVar[int]
    BREAKPOINT_MARKER_FIELD_NUMBER: _ClassVar[int]
    STANDARD_COMPLIANCE_FIELD_NUMBER: _ClassVar[int]
    supported: bool
    ttl_seconds: int
    control_parameter: str
    breakpoint_marker: str
    standard_compliance: bool
    def __init__(self, supported: bool = ..., ttl_seconds: _Optional[int] = ..., control_parameter: _Optional[str] = ..., breakpoint_marker: _Optional[str] = ..., standard_compliance: bool = ...) -> None: ...

class CapabilitiesDeclaration(_message.Message):
    __slots__ = ("context_window", "modalities", "tool_calling", "streaming")
    CONTEXT_WINDOW_FIELD_NUMBER: _ClassVar[int]
    MODALITIES_FIELD_NUMBER: _ClassVar[int]
    TOOL_CALLING_FIELD_NUMBER: _ClassVar[int]
    STREAMING_FIELD_NUMBER: _ClassVar[int]
    context_window: int
    modalities: _containers.RepeatedScalarFieldContainer[str]
    tool_calling: ToolCallingDeclaration
    streaming: StreamingDeclaration
    def __init__(self, context_window: _Optional[int] = ..., modalities: _Optional[_Iterable[str]] = ..., tool_calling: _Optional[_Union[ToolCallingDeclaration, _Mapping]] = ..., streaming: _Optional[_Union[StreamingDeclaration, _Mapping]] = ...) -> None: ...

class ToolCallingDeclaration(_message.Message):
    __slots__ = ("supported", "schema_format")
    SUPPORTED_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FORMAT_FIELD_NUMBER: _ClassVar[int]
    supported: bool
    schema_format: str
    def __init__(self, supported: bool = ..., schema_format: _Optional[str] = ...) -> None: ...

class StreamingDeclaration(_message.Message):
    __slots__ = ("supported", "protocol")
    SUPPORTED_FIELD_NUMBER: _ClassVar[int]
    PROTOCOL_FIELD_NUMBER: _ClassVar[int]
    supported: bool
    protocol: str
    def __init__(self, supported: bool = ..., protocol: _Optional[str] = ...) -> None: ...

class EndpointDeclaration(_message.Message):
    __slots__ = ("type", "region", "base_url", "tls_version", "auth_method", "priority", "provision_url", "documentation_url")
    TYPE_FIELD_NUMBER: _ClassVar[int]
    REGION_FIELD_NUMBER: _ClassVar[int]
    BASE_URL_FIELD_NUMBER: _ClassVar[int]
    TLS_VERSION_FIELD_NUMBER: _ClassVar[int]
    AUTH_METHOD_FIELD_NUMBER: _ClassVar[int]
    PRIORITY_FIELD_NUMBER: _ClassVar[int]
    PROVISION_URL_FIELD_NUMBER: _ClassVar[int]
    DOCUMENTATION_URL_FIELD_NUMBER: _ClassVar[int]
    type: str
    region: str
    base_url: str
    tls_version: str
    auth_method: str
    priority: int
    provision_url: str
    documentation_url: str
    def __init__(self, type: _Optional[str] = ..., region: _Optional[str] = ..., base_url: _Optional[str] = ..., tls_version: _Optional[str] = ..., auth_method: _Optional[str] = ..., priority: _Optional[int] = ..., provision_url: _Optional[str] = ..., documentation_url: _Optional[str] = ...) -> None: ...

class ComplianceDeclaration(_message.Message):
    __slots__ = ("data_processing", "content_filtering", "data_portability")
    DATA_PROCESSING_FIELD_NUMBER: _ClassVar[int]
    CONTENT_FILTERING_FIELD_NUMBER: _ClassVar[int]
    DATA_PORTABILITY_FIELD_NUMBER: _ClassVar[int]
    data_processing: str
    content_filtering: str
    data_portability: str
    def __init__(self, data_processing: _Optional[str] = ..., content_filtering: _Optional[str] = ..., data_portability: _Optional[str] = ...) -> None: ...

class RateLimits(_message.Message):
    __slots__ = ("tiers",)
    TIERS_FIELD_NUMBER: _ClassVar[int]
    tiers: _containers.RepeatedCompositeFieldContainer[RateLimitTier]
    def __init__(self, tiers: _Optional[_Iterable[_Union[RateLimitTier, _Mapping]]] = ...) -> None: ...

class RateLimitTier(_message.Message):
    __slots__ = ("tier_name", "description", "rpm", "tpm", "rpd", "tpd", "max_concurrent", "reset_schedule", "reset_time_utc", "models_included", "peak_hours", "model_overrides", "header_format")
    class ModelOverridesEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: ModelRateLimit
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[ModelRateLimit, _Mapping]] = ...) -> None: ...
    TIER_NAME_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    RPM_FIELD_NUMBER: _ClassVar[int]
    TPM_FIELD_NUMBER: _ClassVar[int]
    RPD_FIELD_NUMBER: _ClassVar[int]
    TPD_FIELD_NUMBER: _ClassVar[int]
    MAX_CONCURRENT_FIELD_NUMBER: _ClassVar[int]
    RESET_SCHEDULE_FIELD_NUMBER: _ClassVar[int]
    RESET_TIME_UTC_FIELD_NUMBER: _ClassVar[int]
    MODELS_INCLUDED_FIELD_NUMBER: _ClassVar[int]
    PEAK_HOURS_FIELD_NUMBER: _ClassVar[int]
    MODEL_OVERRIDES_FIELD_NUMBER: _ClassVar[int]
    HEADER_FORMAT_FIELD_NUMBER: _ClassVar[int]
    tier_name: str
    description: str
    rpm: int
    tpm: int
    rpd: int
    tpd: int
    max_concurrent: int
    reset_schedule: str
    reset_time_utc: str
    models_included: _containers.RepeatedScalarFieldContainer[str]
    peak_hours: PeakHours
    model_overrides: _containers.MessageMap[str, ModelRateLimit]
    header_format: RateLimitHeaders
    def __init__(self, tier_name: _Optional[str] = ..., description: _Optional[str] = ..., rpm: _Optional[int] = ..., tpm: _Optional[int] = ..., rpd: _Optional[int] = ..., tpd: _Optional[int] = ..., max_concurrent: _Optional[int] = ..., reset_schedule: _Optional[str] = ..., reset_time_utc: _Optional[str] = ..., models_included: _Optional[_Iterable[str]] = ..., peak_hours: _Optional[_Union[PeakHours, _Mapping]] = ..., model_overrides: _Optional[_Mapping[str, ModelRateLimit]] = ..., header_format: _Optional[_Union[RateLimitHeaders, _Mapping]] = ...) -> None: ...

class PeakHours(_message.Message):
    __slots__ = ("enabled", "timezone", "start", "end", "rpm_during_peak")
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    TIMEZONE_FIELD_NUMBER: _ClassVar[int]
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    RPM_DURING_PEAK_FIELD_NUMBER: _ClassVar[int]
    enabled: bool
    timezone: str
    start: str
    end: str
    rpm_during_peak: int
    def __init__(self, enabled: bool = ..., timezone: _Optional[str] = ..., start: _Optional[str] = ..., end: _Optional[str] = ..., rpm_during_peak: _Optional[int] = ...) -> None: ...

class ModelRateLimit(_message.Message):
    __slots__ = ("rpm", "tpm")
    RPM_FIELD_NUMBER: _ClassVar[int]
    TPM_FIELD_NUMBER: _ClassVar[int]
    rpm: int
    tpm: int
    def __init__(self, rpm: _Optional[int] = ..., tpm: _Optional[int] = ...) -> None: ...

class RateLimitHeaders(_message.Message):
    __slots__ = ("remaining_requests", "remaining_tokens", "reset_time_sec")
    REMAINING_REQUESTS_FIELD_NUMBER: _ClassVar[int]
    REMAINING_TOKENS_FIELD_NUMBER: _ClassVar[int]
    RESET_TIME_SEC_FIELD_NUMBER: _ClassVar[int]
    remaining_requests: str
    remaining_tokens: str
    reset_time_sec: str
    def __init__(self, remaining_requests: _Optional[str] = ..., remaining_tokens: _Optional[str] = ..., reset_time_sec: _Optional[str] = ...) -> None: ...

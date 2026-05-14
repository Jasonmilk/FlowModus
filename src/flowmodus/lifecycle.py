import os
import time
import signal
import asyncio
from dataclasses import dataclass, field
from typing import Dict, List, Optional

from flowmodus.schemas.routing_pb2 import RawRequest, RoutingDecision, UserConstraints
from flowmodus.schemas.supplier_pb2 import SupplierDeclaration
from flowmodus.data_plane.telemetry import DeviationSnapshot, TelemetryCollector
from flowmodus.data_plane.pipeline import (
    Layer1Normalizer,
    Layer2Registry,
    Layer3CostEstimator,
    Layer4HardFilter,
    Layer5Scorer,
)
from flowmodus.config.bias import BiasConfig


class SidecarState:
    INIT = "INIT"
    BOOTSTRAP = "BOOTSTRAP"
    READY = "READY"
    DEGRADED = "DEGRADED"
    FREEZING = "FREEZING"
    SHUTTING_DOWN = "SHUTTING_DOWN"
    DEAD = "DEAD"


@dataclass
class SidecarConfig:
    proxy_host: str = field(default_factory=lambda: os.getenv("FLOWMODUS_PROXY_HOST", "127.0.0.1"))
    proxy_port: int = field(default_factory=lambda: int(os.getenv("FLOWMODUS_PROXY_PORT", "8080")))
    shutdown_timeout_sec: int = field(default_factory=lambda: int(os.getenv("FLOWMODUS_SHUTDOWN_TIMEOUT", "30")))
    max_retry_count: int = field(default_factory=lambda: int(os.getenv("FLOWMODUS_MAX_RETRY_COUNT", "3")))
    data_sharing: bool = field(default_factory=lambda: os.getenv("FLOWMODUS_DATA_SHARING", "off").lower() == "on")


class Sidecar:
    """Lifecycle manager for a single FlowModus sidecar instance."""

    def __init__(self, config: Optional[SidecarConfig] = None) -> None:
        self.config = config or SidecarConfig()
        self.state = SidecarState.INIT
        self.instance_id = self._generate_instance_id()
        self.registry_snapshot: List[SupplierDeclaration] = []
        self.deviation_snapshot = DeviationSnapshot({})
        self.telemetry_collector = TelemetryCollector()
        self.bias_config = BiasConfig()
        self.user_constraints = UserConstraints()
        self.health_states: Dict[str, str] = {}
        self.health_timestamps: Dict[str, int] = {}
        self.tokenizer_ratios: Dict[str, float] = {"default": 1.0}
        self._inflight_count = 0
        self._shutdown_event: Optional[asyncio.Event] = None

    @staticmethod
    def _generate_instance_id() -> str:
        from cryptography.hazmat.primitives.asymmetric import ed25519
        from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
        import hashlib
        temp_key = ed25519.Ed25519PrivateKey.generate()
        public_bytes = temp_key.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
        return hashlib.sha256(public_bytes).hexdigest()[:16]

    @property
    def current_minute(self) -> int:
        return int(time.time() // 60)

    def get_endpoint_url(self, supplier_id: str, model_id: str) -> Optional[str]:
        """Return the complete endpoint URL as declared in the registry."""
        for supplier in self.registry_snapshot:
            if supplier.supplier_id == supplier_id:
                for model in supplier.models:
                    if model.model_id == model_id:
                        if supplier.endpoints:
                            return supplier.endpoints[0].base_url.rstrip("/")
        return None

    def _resolve_supplier_id(self, model_id: str) -> str:
        for supplier in self.registry_snapshot:
            for model in supplier.models:
                if model.model_id == model_id:
                    return supplier.supplier_id
        return ""

    def _resolve_endpoint_url(self, model_id: str) -> Optional[str]:
        """Resolve the complete endpoint URL for a concrete model_id."""
        for supplier in self.registry_snapshot:
            for model in supplier.models:
                if model.model_id == model_id:
                    if supplier.endpoints:
                        return supplier.endpoints[0].base_url.rstrip("/")
        return None

    def create_eligible_supplier(self, estimate, endpoint_url: str):
        from flowmodus.schemas.routing_pb2 import EligibleSupplier
        return EligibleSupplier(
            supplier_id=estimate.supplier_id,
            model_id=estimate.model_id,
            endpoint_url=endpoint_url,
            score=0.0,
            claim_deviation=0.0,
            cost=estimate,
        )

    def translate_cache_breakpoints(self, body: dict, supplier_id: str) -> dict:
        return body

    def extract_usage_from_response(self, response) -> dict:
        try:
            if hasattr(response, 'json'):
                data = response.json()
                return data.get("usage", {})
        except Exception:
            pass
        return {}

    def track_inflight_request(self) -> None:
        self._inflight_count += 1

    def untrack_inflight_request(self) -> None:
        if self._inflight_count > 0:
            self._inflight_count -= 1

    async def drain_inflight_requests(self) -> None:
        deadline = time.monotonic() + self.config.shutdown_timeout_sec
        while self._inflight_count > 0 and time.monotonic() < deadline:
            await asyncio.sleep(0.1)

    # ----- routing dispatch -----
    def resolve_routing_decision(
        self,
        raw_request: RawRequest,
        model_param: str,
        headers: dict,
    ) -> RoutingDecision:
        if model_param and not model_param.startswith("group:") and model_param != "auto":
            return self._manual_decision(model_param)
        if model_param.startswith("group:"):
            group_name = model_param[len("group:"):]
            return self._group_decision(raw_request, group_name)
        return self._auto_decision(raw_request)

    def _manual_decision(self, model_id: str) -> RoutingDecision:
        endpoint_url = self._resolve_endpoint_url(model_id)
        if not endpoint_url:
            raise ValueError(f"Model '{model_id}' not found in registry")
        return RoutingDecision(
            supplier_id=self._resolve_supplier_id(model_id),
            model_id=model_id,
            endpoint_url=endpoint_url,
            estimated_cost_usd=0.0,
            request_id=model_id,
        )

    def _group_decision(self, raw_request: RawRequest, group_name: str) -> RoutingDecision:
        return self._auto_decision(raw_request)

    def _auto_decision(self, raw_request: RawRequest) -> RoutingDecision:
        normalized = Layer1Normalizer.normalize_request(raw_request, self.tokenizer_ratios)
        layer2 = Layer2Registry(self.registry_snapshot)
        suppliers = layer2.get_eligible_suppliers(normalized)
        layer3 = Layer3CostEstimator(self.deviation_snapshot)
        estimates = layer3.estimate_all(normalized, suppliers)
        layer4 = Layer4HardFilter(
            self.user_constraints,
            self.deviation_snapshot,
            self.bias_config,
            self.health_states,
            self.health_timestamps,
            self.instance_id,
            self.current_minute,
        )
        filtered = layer4.filter(estimates)
        eligible = []
        for estimate in filtered:
            endpoint_url = self.get_endpoint_url(estimate.supplier_id, estimate.model_id)
            if endpoint_url:
                eligible.append(self.create_eligible_supplier(estimate, endpoint_url))
        layer5 = Layer5Scorer(
            normalized.agent_role,
            self.instance_id,
            normalized.prompt_hash,
            self.bias_config,
        )
        return layer5.select(eligible)

    # ----- startup / shutdown -----
    async def start(self) -> None:
        if self.state != SidecarState.INIT:
            return
        self.state = SidecarState.BOOTSTRAP
        local_registry_path = os.getenv("FLOWMODUS_LOCAL_REGISTRY")
        if local_registry_path:
            import json
            from google.protobuf.json_format import Parse
            with open(local_registry_path, "r") as f:
                data = json.load(f)
                for raw in data.get("suppliers", []):
                    supplier = Parse(json.dumps(raw), SupplierDeclaration())
                    self.registry_snapshot.append(supplier)
        self.state = SidecarState.READY

    async def shutdown(self, sig: signal.Signals) -> None:
        if self.state in (SidecarState.SHUTTING_DOWN, SidecarState.DEAD):
            return
        self.state = SidecarState.SHUTTING_DOWN
        await self.drain_inflight_requests()
        self.telemetry_collector.flush()
        self.state = SidecarState.DEAD

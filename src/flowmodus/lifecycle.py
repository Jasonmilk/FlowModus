import os
import signal
import time
import asyncio
from dataclasses import dataclass, field
from enum import auto
from typing import Dict, List, Optional

from flowmodus.schemas.routing_pb2 import UserConstraints, CostEstimate, EligibleSupplier
from flowmodus.schemas.supplier_pb2 import SupplierDeclaration
from flowmodus.data_plane.telemetry import DeviationSnapshot, TelemetryCollector
from flowmodus.config.bias import BiasConfig


# ---------- Minimal state enum ----------
class SidecarState:
    INIT = auto()
    BOOTSTRAP = auto()
    READY = auto()
    DEGRADED = auto()
    FREEZING = auto()
    SHUTTING_DOWN = auto()
    DEAD = auto()


# ---------- Configuration ----------
@dataclass
class SidecarConfig:
    """User-configurable parameters for the sidecar. All values can be set via
    environment variables; defaults are provided for local development."""

    proxy_host: str = field(default_factory=lambda: os.getenv("FLOWMODUS_PROXY_HOST", "127.0.0.1"))
    proxy_port: int = field(default_factory=lambda: int(os.getenv("FLOWMODUS_PROXY_PORT", "8080")))
    shutdown_timeout_sec: int = field(default_factory=lambda: int(os.getenv("FLOWMODUS_SHUTDOWN_TIMEOUT", "30")))
    max_retry_count: int = field(default_factory=lambda: int(os.getenv("FLOWMODUS_MAX_RETRY_COUNT", "3")))
    data_sharing: bool = field(default_factory=lambda: os.getenv("FLOWMODUS_DATA_SHARING", "off").lower() == "on")


# ---------- Sidecar ----------
class Sidecar:
    """Lifecycle manager for a single FlowModus sidecar instance."""

    def __init__(self, config: Optional[SidecarConfig] = None) -> None:
        self.config = config or SidecarConfig()
        self.state = SidecarState.INIT

        # Identity
        self.instance_id = self._generate_instance_id()

        # Registry snapshot (set by control plane)
        self.registry_snapshot: List[SupplierDeclaration] = []

        # Telemetry
        self.deviation_snapshot = DeviationSnapshot({})
        self.telemetry_collector = TelemetryCollector()

        # User preferences (loaded from bias.yaml at boot)
        self.bias_config = BiasConfig()
        self.user_constraints = UserConstraints()

        # Health tracking
        self.health_states: Dict[str, str] = {}
        self.health_timestamps: Dict[str, int] = {}

        # Tokenizer ratios for normalization
        self.tokenizer_ratios: Dict[str, float] = {"default": 1.0}

        # Internal bookkeeping
        self._inflight_count = 0
        self._shutdown_event: Optional[asyncio.Event] = None

    # ----- identity -----
    @staticmethod
    def _generate_instance_id() -> str:
        """Generate a unique instance id for this sidecar process."""
        from cryptography.hazmat.primitives.asymmetric import ed25519
        temp_key = ed25519.Ed25519PrivateKey.generate()
        from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
        public_bytes = temp_key.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
        import hashlib
        return hashlib.sha256(public_bytes).hexdigest()[:16]

    # ----- state helpers -----
    @property
    def current_minute(self) -> int:
        return int(time.time() // 60)

    # ----- endpoint lookup -----
    def get_endpoint_url(self, supplier_id: str, model_id: str) -> Optional[str]:
        """Return the first endpoint URL for the given supplier/model."""
        for supplier in self.registry_snapshot:
            if supplier.supplier_id == supplier_id:
                for model in supplier.models:
                    if model.model_id == model_id:
                        if supplier.endpoints:
                            return supplier.endpoints[0].base_url
        return None

    # ----- eligible supplier construction -----
    def create_eligible_supplier(self, estimate: CostEstimate, endpoint_url: str) -> EligibleSupplier:
        """Build an EligibleSupplier message from a CostEstimate."""
        return EligibleSupplier(
            supplier_id=estimate.supplier_id,
            model_id=estimate.model_id,
            endpoint_url=endpoint_url,
            score=0.0,  # will be re-scored in Layer 5
            claim_deviation=0.0,  # placeholder, Layer 4 already checked tolerance
            cost=estimate,
        )

    # ----- cache breakpoint translation -----
    def translate_cache_breakpoints(self, body: dict, supplier_id: str) -> dict:
        """Adapt generic cache breakpoints to the supplier's format."""
        # For now, return the body unchanged; real translation requires
        # per-supplier mapping that will be loaded from the registry.
        return body

    # ----- usage extraction -----
    def extract_usage_from_response(self, response) -> dict:
        """Extract usage information from a successful HTTP response."""
        try:
            if hasattr(response, 'json'):
                data = response.json()
                return data.get("usage", {})
        except Exception:
            pass
        return {}

    # ----- inflight request tracking -----
    def track_inflight_request(self) -> None:
        self._inflight_count += 1

    def untrack_inflight_request(self) -> None:
        if self._inflight_count > 0:
            self._inflight_count -= 1

    async def drain_inflight_requests(self) -> None:
        """Wait until all inflight requests are finished or timeout."""
        deadline = time.monotonic() + self.config.shutdown_timeout_sec
        while self._inflight_count > 0 and time.monotonic() < deadline:
            await asyncio.sleep(0.1)

    # ----- startup / shutdown -----
    async def start(self) -> None:
        """Bootstrap the sidecar (control plane) and transition to READY."""
        if self.state != SidecarState.INIT:
            return

        self.state = SidecarState.BOOTSTRAP

        # Simulate bootstrap; in real implementation the control plane
        # would pull the registry, verify signature, and populate snapshots.
        # For now we assume the caller has already set registry_snapshot.

        self.state = SidecarState.READY

    async def shutdown(self, sig: signal.Signals) -> None:
        """Graceful shutdown triggered by an OS signal."""
        if self.state in (SidecarState.SHUTTING_DOWN, SidecarState.DEAD):
            return

        self.state = SidecarState.SHUTTING_DOWN
        await self.drain_inflight_requests()
        # Persist telemetry before dying
        self.telemetry_collector.flush()
        self.state = SidecarState.DEAD

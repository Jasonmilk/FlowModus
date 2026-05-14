import asyncio
import signal
from enum import Enum
from typing import Optional, Dict, Any

from flowmodus.control_plane import IpfsClient, verify_registry, canonicalize_json, AntiCorruption
from flowmodus.gossip import GossipClient


class SidecarState(Enum):
    INIT = "INIT"
    BOOTSTRAP = "BOOTSTRAP"
    READY = "READY"
    DEGRADED = "DEGRADED"
    FREEZING = "FREEZING"
    SHUTTING_DOWN = "SHUTTING_DOWN"
    DEAD = "DEAD"


class SidecarConfig:
    """Sidecar configuration."""
    shutdown_timeout_sec: int = 30
    registry_refresh_interval_sec: int = 3600
    data_sharing_enabled: bool = False
    max_retry_count: int = 3
    
    def __init__(self, **kwargs: Any):
        for key, value in kwargs.items():
            if hasattr(self, key):
                setattr(self, key, value)


class Sidecar:
    """Main FlowModus Sidecar class."""
    
    state: SidecarState
    config: SidecarConfig
    instance_id: str
    
    # Control plane
    control_plane_task: Optional[asyncio.Task]
    
    # Data plane
    proxy: Optional["Proxy"]
    telemetry_collector: Optional["TelemetryCollector"]
    
    # Gossip
    gossip_client: Optional[GossipClient]
    
    # Runtime state
    _inflight_requests: int
    _registry_snapshot: Optional[Any]
    _deviation_snapshot: Optional[Any]
    
    def __init__(self, config: Optional[SidecarConfig] = None):
        self.state = SidecarState.INIT
        self.config = config or SidecarConfig()
        self._inflight_requests = 0
        self._registry_snapshot = None
        self._deviation_snapshot = None
        
        # Generate instance ID (local mutation salt)
        # Will be replaced with proper Ed25519 key hash in full implementation
        import secrets
        self.instance_id = secrets.token_hex(16)
    
    async def start(self) -> None:
        """Start the Sidecar."""
        if self.state != SidecarState.INIT:
            raise RuntimeError(f"Cannot start from state {self.state}")
        
        self.state = SidecarState.BOOTSTRAP
        
        # Setup signal handlers for graceful shutdown
        loop = asyncio.get_running_loop()
        for sig in (signal.SIGTERM, signal.SIGINT):
            loop.add_signal_handler(
                sig,
                lambda s=sig: asyncio.create_task(self._shutdown(s))
            )
        
        # Bootstrap control plane
        await self._bootstrap_control_plane()
        
        # Initialize data plane
        await self._init_data_plane()
        
        # Initialize gossip if enabled
        if self.config.data_sharing_enabled:
            await self._init_gossip()
        
        # Start periodic registry refresh
        self.control_plane_task = asyncio.create_task(self._periodic_refresh())
        
        # Mark as ready
        self.state = SidecarState.READY
    
    async def _bootstrap_control_plane(self) -> None:
        """Bootstrap control plane: load and verify registry."""
        # TODO: Full implementation
        # 1. Load local config
        # 2. Pull registry from IPNS
        # 3. Verify signatures
        # 4. Canonicalize
        # 5. Convert to Protobuf
        # 6. Load cached telemetry
        pass
    
    async def _init_data_plane(self) -> None:
        """Initialize data plane proxy and telemetry."""
        # TODO: Full implementation
        self.telemetry_collector = TelemetryCollector()
        self.proxy = Proxy(self)
    
    async def _init_gossip(self) -> None:
        """Initialize gossip client."""
        # TODO: Full implementation
        self.gossip_client = GossipClient(self.instance_id)
    
    async def _periodic_refresh(self) -> None:
        """Periodic registry refresh task."""
        while self.state not in (SidecarState.SHUTTING_DOWN, SidecarState.DEAD):
            await asyncio.sleep(self.config.registry_refresh_interval_sec)
            if self.state in (SidecarState.READY, SidecarState.DEGRADED):
                try:
                    # Refresh registry
                    await self._bootstrap_control_plane()
                except Exception:
                    # Failed to refresh, use cached version
                    pass
    
    async def _shutdown(self, signal_received: signal.Signals) -> None:
        """Graceful shutdown handler."""
        if self.state == SidecarState.SHUTTING_DOWN:
            return
        
        self.state = SidecarState.SHUTTING_DOWN
        
        try:
            # Wait for inflight requests to drain
            await asyncio.wait_for(
                self._drain_inflight_requests(),
                timeout=self.config.shutdown_timeout_sec
            )
        except asyncio.TimeoutError:
            pass
        
        # Flush telemetry
        if self.telemetry_collector:
            self.telemetry_collector.flush()
        
        # Mark as dead
        self.state = SidecarState.DEAD
    
    async def _drain_inflight_requests(self) -> None:
        """Wait for all inflight requests to complete."""
        while self._inflight_requests > 0:
            await asyncio.sleep(0.1)
    
    def track_inflight_request(self) -> None:
        """Track an incoming request."""
        self._inflight_requests += 1
    
    def untrack_inflight_request(self) -> None:
        """Track a completed request."""
        self._inflight_requests -= 1

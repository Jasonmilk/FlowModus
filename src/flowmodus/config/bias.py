from dataclasses import dataclass, field
from typing import Dict, List, Optional


@dataclass
class SupplierBias:
    """User-defined bias for a specific supplier."""
    bias_score: float = 0.0
    max_cost_per_request: Optional[float] = None
    max_cost_limit: Optional[float] = None
    notes: str = ""


@dataclass
class PriorityGroup:
    """User-defined priority group for channel cascading."""
    name: str
    priority: int
    models: List[str] = field(default_factory=list)


@dataclass
class GroupEndpoint:
    """A single endpoint within a user-defined routing group."""
    id: str
    priority: int = 1
    weight: int = 10


@dataclass
class RoutingGroup:
    """User-defined routing group with prioritized endpoints."""
    description: str = ""
    endpoints: List[GroupEndpoint] = field(default_factory=list)


@dataclass
class Overlay:
    """Local overrides for a supplier's declared values."""
    display_name: Optional[str] = None
    price_input: Optional[float] = None
    price_output: Optional[float] = None
    tags_add: List[str] = field(default_factory=list)


@dataclass
class BiasConfig:
    """Full user bias configuration."""
    supplier_biases: Dict[str, SupplierBias] = field(default_factory=dict)
    priority_groups: List[PriorityGroup] = field(default_factory=list)
    rehabilitation_probability: float = 0.001  # 0.1% default, user-overridable
    rehabilitation_cooldown_seconds: int = 300  # 5 min default, user-overridable
    groups: Dict[str, RoutingGroup] = field(default_factory=dict)
    overlays: Dict[str, Overlay] = field(default_factory=dict)
    declared_tier: Dict[str, str] = field(default_factory=dict)  # supplier_id -> tier_name

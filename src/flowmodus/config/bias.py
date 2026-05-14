from dataclasses import dataclass, field
from typing import List, Dict, Optional


@dataclass
class PriorityGroup:
    name: str
    priority: int
    models: List[str] = field(default_factory=list)


@dataclass
class SupplierBias:
    bias_score: float = 0.0
    max_cost_per_request: Optional[float] = None
    notes: str = ""


@dataclass
class BiasConfig:
    priority_groups: List[PriorityGroup] = field(default_factory=list)
    supplier_biases: Dict[str, SupplierBias] = field(default_factory=dict)

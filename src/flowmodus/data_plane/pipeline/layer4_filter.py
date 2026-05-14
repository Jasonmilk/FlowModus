from typing import List, Dict, Optional

from flowmodus.schemas.routing_pb2 import CostEstimate, UserConstraints
from flowmodus.data_plane.telemetry import DeviationSnapshot
from flowmodus.config.bias import BiasConfig, PriorityGroup


REHABILITATION_PROBABILITY = 0.001  # 0.1%


def should_rehabilitate(supplier_id: str, instance_id: str, time_since_degraded: int) -> bool:
    """
    Check if we should attempt rehabilitation for a degraded supplier.
    Pure function.
    
    Args:
        supplier_id: Supplier ID
        instance_id: Sidecar instance ID
        time_since_degraded: Seconds since supplier was marked degraded
    
    Returns:
        True if rehabilitation should be attempted
    """
    if time_since_degraded < 300:
        return False
    
    # Deterministic seed based on instance, supplier, and current minute
    import time
    current_minute = int(time.time() // 60)
    seed = hash(instance_id + supplier_id + str(current_minute))
    
    return (seed % 1000) == 0


def apply_priority_filter(
    candidates: List[CostEstimate],
    priority_groups: List[PriorityGroup],
    health_states: Dict[str, str],
    instance_id: str,
) -> List[CostEstimate]:
    """
    Apply effective cascading priority filter.
    Pure function.
    
    Args:
        candidates: List of cost estimates
        priority_groups: User priority groups
        health_states: Current health states of suppliers
        instance_id: Sidecar instance ID
    
    Returns:
        Filtered candidates
    """
    if not priority_groups:
        return candidates
    
    # Sort groups by priority descending
    sorted_groups = sorted(priority_groups, key=lambda g: g.priority, reverse=True)
    
    for group in sorted_groups:
        group_candidates = []
        for candidate in candidates:
            if candidate.model_id not in group.models:
                continue
            
            state = health_states.get(candidate.supplier_id, "HEALTHY")
            
            if state == "TERMINAL":
                continue
            
            if state == "DEGRADED":
                # TODO: Get time since degraded
                time_since_degraded = 0
                if not should_rehabilitate(candidate.supplier_id, instance_id, time_since_degraded):
                    continue
            
            group_candidates.append(candidate)
        
        if group_candidates:
            return group_candidates
    
    return candidates


def apply_hard_filters(
    candidates: List[CostEstimate],
    constraints: UserConstraints,
    deviation_snapshot: DeviationSnapshot,
    bias_config: BiasConfig,
    health_states: Dict[str, str],
    instance_id: str,
) -> List[CostEstimate]:
    """
    Apply hard filters to candidates.
    Pure function.
    
    Args:
        candidates: List of cost estimates
        constraints: User constraints
        deviation_snapshot: Claim deviation snapshot
        bias_config: User bias config
        health_states: Current health states
        instance_id: Sidecar instance ID
    
    Returns:
        Filtered candidates
    """
    filtered = []
    
    for candidate in candidates:
        # Max cost filter
        if candidate.estimated_cost_usd > constraints.max_cost_per_request_usd:
            continue
        
        # Verified supplier filter
        if constraints.require_verified_supplier:
            # TODO: Check if supplier is verified
            pass
        
        # Deviation tolerance filter
        deviation = deviation_snapshot.get_deviation(candidate.supplier_id, "billing_accuracy")
        if deviation and abs(deviation.deviation_percent) > constraints.max_claim_deviation_tolerance:
            continue
        
        # Supplier specific max cost from bias
        supplier_bias = bias_config.supplier_biases.get(candidate.supplier_id)
        if supplier_bias and supplier_bias.max_cost_per_request:
            if candidate.estimated_cost_usd > supplier_bias.max_cost_per_request:
                continue
        
        filtered.append(candidate)
    
    # Apply priority cascading filter
    filtered = apply_priority_filter(
        filtered,
        bias_config.priority_groups,
        health_states,
        instance_id
    )
    
    return filtered


class Layer4HardFilter:
    """
    Layer 4: User preferences and hard boundaries.
    Applies hard filters based on user constraints and preferences.
    """
    
    def __init__(
        self,
        constraints: UserConstraints,
        deviation_snapshot: DeviationSnapshot,
        bias_config: BiasConfig,
        health_states: Dict[str, str],
        instance_id: str,
    ):
        self._constraints = constraints
        self._deviation_snapshot = deviation_snapshot
        self._bias_config = bias_config
        self._health_states = health_states
        self._instance_id = instance_id
    
    def filter(self, candidates: List[CostEstimate]) -> List[CostEstimate]:
        """
        Filter candidates based on hard constraints.
        Pure function.
        
        Args:
            candidates: List of cost estimates
        
        Returns:
            Filtered candidates
        """
        return apply_hard_filters(
            candidates,
            self._constraints,
            self._deviation_snapshot,
            self._bias_config,
            self._health_states,
            self._instance_id
        )

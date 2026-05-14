from typing import List, Dict

from flowmodus.schemas.routing_pb2 import CostEstimate, UserConstraints
from flowmodus.data_plane.telemetry import DeviationSnapshot
from flowmodus.config.bias import BiasConfig, PriorityGroup


def should_rehabilitate(
    supplier_id: str,
    instance_id: str,
    time_since_degraded: int,
    current_minute: int,
    rehabilitation_probability: float,
    rehabilitation_cooldown_seconds: int,
) -> bool:
    """
    Check if we should attempt rehabilitation for a degraded supplier.
    Pure function: all inputs are explicit, no side effects.

    Args:
        supplier_id: Supplier identifier
        instance_id: Sidecar instance identifier
        time_since_degraded: Seconds elapsed since the supplier was marked degraded
        current_minute: Current minute as an integer (e.g., int(time.time() // 60))
        rehabilitation_probability: Probability threshold (e.g., 0.001 for 0.1%)
        rehabilitation_cooldown_seconds: Minimum seconds before rehabs can occur
    """
    if time_since_degraded < rehabilitation_cooldown_seconds:
        return False

    seed = hash(instance_id + supplier_id + str(current_minute))
    # Map probability to modulus space: e.g., 0.001 → modulus 1000, check remainder 0
    modulus = int(1.0 / rehabilitation_probability) if rehabilitation_probability > 0 else 1
    return (seed % modulus) == 0


def apply_priority_filter(
    candidates: List[CostEstimate],
    priority_groups: List[PriorityGroup],
    health_states: Dict[str, str],
    health_timestamps: Dict[str, int],
    instance_id: str,
    current_minute: int,
    bias_config: BiasConfig,
) -> List[CostEstimate]:
    """
    Apply effective cascading priority filter.
    Pure function.
    """
    if not priority_groups:
        return candidates

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
                time_since_degraded = int(
                    (current_minute * 60) - health_timestamps.get(candidate.supplier_id, 0)
                )
                if time_since_degraded < 0:
                    time_since_degraded = 0
                if not should_rehabilitate(
                    candidate.supplier_id,
                    instance_id,
                    time_since_degraded,
                    current_minute,
                    bias_config.rehabilitation_probability,
                    bias_config.rehabilitation_cooldown_seconds,
                ):
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
    health_timestamps: Dict[str, int],
    instance_id: str,
    current_minute: int,
) -> List[CostEstimate]:
    """
    Apply hard filters to candidates.
    Pure function.
    """
    filtered = []

    for candidate in candidates:
        if constraints.max_cost_per_request_usd > 0:
            if candidate.estimated_cost_usd > constraints.max_cost_per_request_usd:
                continue

        if constraints.require_verified_supplier:
            # Supplier verification status is read from the registry at request time
            # This is passed in via the filtered candidates list already.
            pass

        deviation = deviation_snapshot.get_deviation(
            candidate.supplier_id, "billing_accuracy"
        )
        if (
            deviation
            and constraints.max_claim_deviation_tolerance > 0
            and abs(deviation.deviation_percent)
            > constraints.max_claim_deviation_tolerance
        ):
            continue

        supplier_bias = bias_config.supplier_biases.get(candidate.supplier_id)
        if (
            supplier_bias
            and supplier_bias.max_cost_per_request
            and candidate.estimated_cost_usd > supplier_bias.max_cost_per_request
        ):
            continue

        filtered.append(candidate)

    filtered = apply_priority_filter(
        filtered,
        bias_config.priority_groups,
        health_states,
        health_timestamps,
        instance_id,
        current_minute,
        bias_config,
    )

    return filtered


class Layer4HardFilter:
    """Layer 4: User preferences and hard boundaries."""

    def __init__(
        self,
        constraints: UserConstraints,
        deviation_snapshot: DeviationSnapshot,
        bias_config: BiasConfig,
        health_states: Dict[str, str],
        health_timestamps: Dict[str, int],
        instance_id: str,
        current_minute: int,
    ):
        self._constraints = constraints
        self._deviation_snapshot = deviation_snapshot
        self._bias_config = bias_config
        self._health_states = health_states
        self._health_timestamps = health_timestamps
        self._instance_id = instance_id
        self._current_minute = current_minute

    def filter(self, candidates: List[CostEstimate]) -> List[CostEstimate]:
        """Filter candidates based on hard constraints. Pure function."""
        return apply_hard_filters(
            candidates,
            self._constraints,
            self._deviation_snapshot,
            self._bias_config,
            self._health_states,
            self._health_timestamps,
            self._instance_id,
            self._current_minute,
        )

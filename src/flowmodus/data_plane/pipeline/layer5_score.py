import hashlib
from typing import List, Tuple

from flowmodus.schemas.routing_pb2 import EligibleSupplier, RoutingDecision, CostEstimate
from flowmodus.config.bias import BiasConfig


def _deterministic_hash(instance_id: str, request_hash: str) -> int:
    """
    Generate a deterministic integer seed from instance_id and request_hash.
    Pure function.
    """
    combined = f"{instance_id}:{request_hash}"
    digest = hashlib.sha256(combined.encode('utf-8')).digest()
    return int.from_bytes(digest[:8], byteorder='big')


def _softmax(scores: List[float]) -> List[float]:
    """
    Compute softmax probabilities from a list of scores.
    Pure function.
    """
    if not scores:
        return []
    # Shift by max for numerical stability
    max_score = max(scores)
    exp_scores = [pow(2.718281828459045, s - max_score) for s in scores]
    total = sum(exp_scores)
    if total == 0:
        # All scores extremely negative, fall back to uniform
        return [1.0 / len(scores)] * len(scores)
    return [e / total for e in exp_scores]


def _weighted_sample(
    candidates: List[EligibleSupplier],
    probabilities: List[float],
    seed: int,
) -> EligibleSupplier:
    """
    Select a candidate based on probability distribution using a deterministic seed.
    Pure function: same seed + same candidates → same result.
    """
    if len(candidates) == 1:
        return candidates[0]
    
    # Use seed to deterministically pick a threshold in [0, 1)
    normalized_seed = (seed % 1000000) / 1000000.0
    
    cumulative = 0.0
    for i, prob in enumerate(probabilities):
        cumulative += prob
        if normalized_seed < cumulative:
            return candidates[i]
    
    # Fallback: return the last candidate (shouldn't happen with valid probabilities)
    return candidates[-1]


def score_and_entropy_sample(
    candidates: List[EligibleSupplier],
    agent_role: str,
    instance_id: str,
    request_hash: str,
    bias_config: BiasConfig,
) -> RoutingDecision:
    """
    Score candidates and perform entropy-weighted routing.
    Pure function.

    Local determinism: same node + same request → same seed → same output.
    Global de-correlation: different instance_id → different seed → different sampling.
    """
    if not candidates:
        raise ValueError("At least one candidate is required")

    # Score each candidate with bias applied
    scored: List[Tuple[float, EligibleSupplier]] = []
    for candidate in candidates:
        score = candidate.score

        # Apply user bias
        supplier_bias = bias_config.supplier_biases.get(candidate.supplier_id)
        if supplier_bias:
            score += supplier_bias.bias_score

        # Role-specific scoring
        if agent_role and agent_role in candidate.model_id.lower():
            score += 10

        scored.append((score, candidate))

    # Compute probabilities via softmax
    raw_scores = [s for s, _ in scored]
    probabilities = _softmax(raw_scores)

    # Deterministic seed for sampling
    seed = _deterministic_hash(instance_id, request_hash)

    # Entropy-weighted selection
    selected = _weighted_sample(
        [c for _, c in scored],
        probabilities,
        seed,
    )

    # Extract KV cache parameters from the selected candidate's cost estimate
    kv_cache_parameter = ""
    kv_cache_ttl = 0
    if selected.cost.kv_cache_applicable:
        kv_cache_parameter = "cache_control"  # Standard default
        kv_cache_ttl = 300  # Default TTL; will be overridden by supplier-specific settings

    return RoutingDecision(
        supplier_id=selected.supplier_id,
        model_id=selected.model_id,
        endpoint_url=selected.endpoint_url,
        estimated_cost_usd=selected.cost.estimated_cost_usd,
        kv_cache_parameter=kv_cache_parameter,
        kv_cache_ttl=kv_cache_ttl,
        request_id=request_hash,
    )


class Layer5Scorer:
    """Layer 5: Agent intent and dynamic preferences."""

    def __init__(
        self,
        agent_role: str,
        instance_id: str,
        request_hash: str,
        bias_config: BiasConfig,
    ):
        self._agent_role = agent_role
        self._instance_id = instance_id
        self._request_hash = request_hash
        self._bias_config = bias_config

    def select(self, candidates: List[EligibleSupplier]) -> RoutingDecision:
        """Select the final supplier from eligible candidates. Pure function."""
        return score_and_entropy_sample(
            candidates,
            self._agent_role,
            self._instance_id,
            self._request_hash,
            self._bias_config,
        )

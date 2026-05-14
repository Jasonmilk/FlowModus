from typing import List

from flowmodus.schemas.routing_pb2 import EligibleSupplier, RoutingDecision, CostEstimate
from flowmodus.config.bias import BiasConfig


def score_and_entropy_sample(
    candidates: List[EligibleSupplier],
    agent_role: str,
    instance_id: str,
    request_hash: str,
    bias_config: BiasConfig,
) -> RoutingDecision:
    """
    Score candidates and perform entropy-weighted sampling.
    Pure function.
    
    Args:
        candidates: List of eligible suppliers
        agent_role: Agent role from request
        instance_id: Sidecar instance ID
        request_hash: Request hash
        bias_config: User bias config
    
    Returns:
        Final routing decision
    """
    # Score each candidate
    scored = []
    for candidate in candidates:
        score = candidate.score
        
        # Apply supplier bias
        supplier_bias = bias_config.supplier_biases.get(candidate.supplier_id)
        if supplier_bias:
            score += supplier_bias.bias_score
        
        # Role-specific adjustments
        if agent_role == "coder" and "coder" in candidate.model_id:
            score += 20
        
        scored.append((score, candidate))
    
    # Sort by score descending
    scored.sort(reverse=True, key=lambda x: x[0])
    
    # For now, pick the top one (deterministic)
    # TODO: Implement entropy sampling for load balancing
    top_score, top_candidate = scored[0]
    
    return RoutingDecision(
        supplier_id=top_candidate.supplier_id,
        model_id=top_candidate.model_id,
        endpoint_url=top_candidate.endpoint_url,
        estimated_cost_usd=top_candidate.cost.estimated_cost_usd,
        # TODO: KV cache parameters
        kv_cache_parameter="",
        kv_cache_ttl=0,
        request_id=request_hash
    )


class Layer5Scorer:
    """
    Layer 5: Agent intent and dynamic preferences.
    Scores candidates and performs entropy-weighted routing.
    """
    
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
        """
        Select the final supplier from eligible candidates.
        Pure function.
        
        Args:
            candidates: List of eligible suppliers
        
        Returns:
            Routing decision
        """
        return score_and_entropy_sample(
            candidates,
            self._agent_role,
            self._instance_id,
            self._request_hash,
            self._bias_config
        )

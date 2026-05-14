from flowmodus.schemas.supplier_pb2 import SupplierDeclaration, ModelDeclaration
from flowmodus.schemas.routing_pb2 import NormalizedRequest, CostEstimate
from flowmodus.data_plane.telemetry import DeviationSnapshot


def estimate_cost(
    request: NormalizedRequest,
    supplier: SupplierDeclaration,
    model: ModelDeclaration,
    deviation_snapshot: DeviationSnapshot,
) -> CostEstimate:
    """
    Estimate the cost of this request for a specific supplier and model.
    Pure function.
    
    Args:
        request: Normalized request
        supplier: Supplier declaration
        model: Model declaration
        deviation_snapshot: Claim deviation snapshot
    
    Returns:
        Cost estimate
    """
    billing = model.billing
    
    # Base cost calculation
    input_cost = request.estimated_input_tokens * billing.token_input
    output_cost = request.max_output_tokens * billing.token_output
    total_cost = input_cost + output_cost
    
    # Apply currency conversion to USD (simplified for now)
    # TODO: Proper currency conversion
    if billing.currency != "USD":
        # Assume 1:1 for now, will be replaced with real rates
        pass
    
    # Calculate STE
    ste_input = request.ste_input
    ste_output = request.ste_output_estimated
    ste_total = ste_input + ste_output
    
    # Check if KV cache is applicable
    kv_cache_applicable = model.kv_cache.supported
    kv_cache_savings = 0.0
    if kv_cache_applicable:
        # Estimate savings based on cache hit rate
        # TODO: Use historical cache hit data
        pass
    
    return CostEstimate(
        supplier_id=supplier.supplier_id,
        model_id=model.model_id,
        estimated_cost_usd=total_cost,
        ste_total=ste_total,
        kv_cache_applicable=kv_cache_applicable,
        kv_cache_savings_estimate=kv_cache_savings
    )


class Layer3CostEstimator:
    """
    Layer 3: Standardized cost estimation.
    Calculates normalized cost estimates for all eligible suppliers.
    """
    
    def __init__(self, deviation_snapshot: DeviationSnapshot):
        self._deviation_snapshot = deviation_snapshot
    
    def estimate_all(
        self,
        request: NormalizedRequest,
        suppliers: list[SupplierDeclaration],
    ) -> list[CostEstimate]:
        """
        Estimate cost for all eligible suppliers.
        Pure function.
        
        Args:
            request: Normalized request
            suppliers: List of eligible suppliers
        
        Returns:
            List of cost estimates
        """
        estimates = []
        for supplier in suppliers:
            for model in supplier.models:
                # TODO: Filter models that can handle this request
                estimate = estimate_cost(
                    request, supplier, model, self._deviation_snapshot
                )
                estimates.append(estimate)
        return estimates

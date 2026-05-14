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
    Pure function. No currency conversion; amounts are in the supplier's
    declared billing currency (typically USD).
    """
    billing = model.billing

    input_cost = request.estimated_input_tokens * billing.token_input
    output_cost = request.max_output_tokens * billing.token_output
    total_cost = input_cost + output_cost

    ste_input = request.ste_input
    ste_output = request.ste_output_estimated
    ste_total = ste_input + ste_output

    kv_cache_applicable = model.kv_cache.supported
    kv_cache_savings = 0.0
    if kv_cache_applicable:
        cache_stats = deviation_snapshot.get(model.model_id, {})
        hit_rate = cache_stats.get("kv_cache_hit_rate", 0.0)
        if hit_rate > 0:
            kv_cache_savings = total_cost * hit_rate * 0.9

    return CostEstimate(
        supplier_id=supplier.supplier_id,
        model_id=model.model_id,
        estimated_cost_usd=total_cost,
        ste_total=ste_total,
        kv_cache_applicable=kv_cache_applicable,
        kv_cache_savings_estimate=round(kv_cache_savings, 6),
    )


def _model_can_handle_request(
    model: ModelDeclaration, request: NormalizedRequest
) -> bool:
    """Check whether a model can handle the given request. Pure function."""
    if model.capabilities.context_window > 0:
        estimated_total = request.estimated_input_tokens + request.max_output_tokens
        if estimated_total > model.capabilities.context_window:
            return False
    return True


class Layer3CostEstimator:
    """Layer 3: Standardized cost estimation. Pure function."""

    def __init__(self, deviation_snapshot: DeviationSnapshot):
        self._deviation_snapshot = deviation_snapshot

    def estimate_all(
        self,
        request: NormalizedRequest,
        suppliers: list[SupplierDeclaration],
    ) -> list[CostEstimate]:
        """Estimate cost for all eligible suppliers. Pure function."""
        estimates = []
        for supplier in suppliers:
            for model in supplier.models:
                if not _model_can_handle_request(model, request):
                    continue
                estimate = estimate_cost(
                    request, supplier, model, self._deviation_snapshot
                )
                estimates.append(estimate)
        return estimates

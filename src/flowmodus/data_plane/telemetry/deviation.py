from typing import Dict, Optional, List
from flowmodus.schemas.metrics_pb2 import ClaimDeviation


def calculate_deviation(claimed: float, actual: float) -> float:
    """
    Calculate claim deviation percentage.
    Pure function.
    
    Args:
        claimed: Claimed value from supplier declaration
        actual: Actual measured value
    
    Returns:
        Deviation percentage
    """
    if claimed == 0.0:
        if actual == 0.0:
            return 0.0
        return abs(actual) * 100.0
    return (actual - claimed) / claimed * 100.0


SETTLEMENT_WINDOW_SIZE = 100
SETTLEMENT_ALERT_THRESHOLD = 10.0  # Percentage


def calculate_settlement_deviation(window: List[float]) -> float:
    """
    Calculate weighted average settlement deviation over sliding window.
    Newer samples have higher weight.
    Pure function.
    
    Args:
        window: List of deviation samples
    
    Returns:
        Weighted average deviation
    """
    if not window:
        return 0.0
    
    total_weight = 0.0
    weighted_sum = 0.0
    for i, deviation in enumerate(window):
        weight = i + 1
        weighted_sum += deviation * weight
        total_weight += weight
    
    return weighted_sum / total_weight


class DeviationSnapshot:
    """
    Immutable snapshot of claim deviations for fast lookup.
    Injected into pipeline by control plane.
    """
    
    def __init__(self, deviations: Dict[str, Dict[str, ClaimDeviation]]):
        # deviations[supplier_id][metric_name] = ClaimDeviation
        self._deviations = deviations
    
    def get_deviation(self, supplier_id: str, metric_name: str) -> Optional[ClaimDeviation]:
        """
        Get deviation for a supplier and metric.
        O(1) lookup.
        
        Args:
            supplier_id: Supplier ID
            metric_name: Metric name
        
        Returns:
            Claim deviation or None if not available
        """
        supplier_deviations = self._deviations.get(supplier_id)
        if not supplier_deviations:
            return None
        return supplier_deviations.get(metric_name)

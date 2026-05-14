from typing import List

from flowmodus.schemas.supplier_pb2 import SupplierDeclaration
from flowmodus.schemas.routing_pb2 import NormalizedRequest


class Layer2Registry:
    """
    Layer 2: Supplier raw model lookup.
    Provides access to the verified supplier registry snapshot.
    """
    
    def __init__(self, registry_snapshot: List[SupplierDeclaration]):
        self._registry = registry_snapshot
    
    def get_eligible_suppliers(self, request: NormalizedRequest) -> List[SupplierDeclaration]:
        """
        Get all suppliers that can potentially handle this request.
        Pure function.
        
        Args:
            request: Normalized request
        
        Returns:
            List of eligible suppliers
        """
        # TODO: Filter suppliers by model capability, agent role, etc.
        return list(self._registry)

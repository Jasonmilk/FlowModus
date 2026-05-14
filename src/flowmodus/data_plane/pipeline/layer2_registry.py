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

    def get_eligible_suppliers(
        self, request: NormalizedRequest
    ) -> List[SupplierDeclaration]:
        """
        Get suppliers that can potentially handle this request.
        Pure function: filters by agent role if specified, otherwise returns all.
        """
        if not request.agent_role:
            return list(self._registry)

        # Prefer suppliers whose models declare this agent role
        matched = []
        for supplier in self._registry:
            for model in supplier.models:
                if request.agent_role in model.agent_roles:
                    matched.append(supplier)
                    break  # One matching model is enough

        # If no supplier declares this role, return all (fallback to Layer 3/4 filtering)
        return matched if matched else list(self._registry)

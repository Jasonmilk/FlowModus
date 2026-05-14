from google.protobuf.json_format import ParseDict
from flowmodus.schemas.registry_pb2 import RegistryPackage
from flowmodus.schemas.supplier_pb2 import SupplierDeclaration


class AntiCorruption:
    """
    Anti-Corruption Layer.
    Converts external JSON data into type-safe Protobuf messages.
    """
    
    @staticmethod
    def parse_registry_package(json_data: dict) -> RegistryPackage:
        """
        Parse JSON registry package into Protobuf message.
        
        Args:
            json_data: JSON data from IPNS
        
        Returns:
            Type-safe RegistryPackage message
        """
        return ParseDict(json_data, RegistryPackage())
    
    @staticmethod
    def parse_supplier_declaration(json_data: dict) -> SupplierDeclaration:
        """
        Parse JSON supplier declaration into Protobuf message.
        
        Args:
            json_data: JSON supplier data
        
        Returns:
            Type-safe SupplierDeclaration message
        """
        return ParseDict(json_data, SupplierDeclaration())

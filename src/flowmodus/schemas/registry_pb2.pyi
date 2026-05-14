import supplier_pb2 as _supplier_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RegistryPackage(_message.Message):
    __slots__ = ("version", "published_at_unix", "suppliers", "signatures")
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PUBLISHED_AT_UNIX_FIELD_NUMBER: _ClassVar[int]
    SUPPLIERS_FIELD_NUMBER: _ClassVar[int]
    SIGNATURES_FIELD_NUMBER: _ClassVar[int]
    version: str
    published_at_unix: int
    suppliers: _containers.RepeatedCompositeFieldContainer[_supplier_pb2.SupplierDeclaration]
    signatures: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, version: _Optional[str] = ..., published_at_unix: _Optional[int] = ..., suppliers: _Optional[_Iterable[_Union[_supplier_pb2.SupplierDeclaration, _Mapping]]] = ..., signatures: _Optional[_Iterable[str]] = ...) -> None: ...

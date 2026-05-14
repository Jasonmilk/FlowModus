from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class GossipMessage(_message.Message):
    __slots__ = ("instance_id", "timestamp_unix", "message_type", "target_id", "payload")
    INSTANCE_ID_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_UNIX_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_TYPE_FIELD_NUMBER: _ClassVar[int]
    TARGET_ID_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    instance_id: str
    timestamp_unix: int
    message_type: str
    target_id: str
    payload: str
    def __init__(self, instance_id: _Optional[str] = ..., timestamp_unix: _Optional[int] = ..., message_type: _Optional[str] = ..., target_id: _Optional[str] = ..., payload: _Optional[str] = ...) -> None: ...

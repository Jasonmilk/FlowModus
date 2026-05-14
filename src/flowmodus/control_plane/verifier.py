from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.exceptions import InvalidSignature


# Protocol root public key (compiled-in constant)
# This is the public part of the offline-generated root key
PROTOCOL_ROOT_PUBLIC_KEY_BYTES: bytes = b''  # TODO: Fill with actual public key


def verify_registry(
    registry_bytes: bytes,
    signature_bytes: bytes,
) -> bool:
    """
    Verify registry signature using Ed25519.
    Pure function.
    
    Args:
        registry_bytes: Raw registry bytes
        signature_bytes: Signature bytes
    
    Returns:
        True if signature is valid
    """
    try:
        public_key = ed25519.Ed25519PublicKey.from_public_bytes(PROTOCOL_ROOT_PUBLIC_KEY_BYTES)
        public_key.verify(signature_bytes, registry_bytes)
        return True
    except InvalidSignature:
        return False


def verify_multisig(
    registry_bytes: bytes,
    signatures: list[bytes],
    council_keys: list[bytes],
    threshold: int,
) -> bool:
    """
    Verify multisig registry signature.
    Pure function.
    
    Args:
        registry_bytes: Raw registry bytes
        signatures: List of signatures
        council_keys: List of council public keys
        threshold: Minimum required valid signatures
    
    Returns:
        True if threshold of valid signatures is met
    """
    valid_count = 0
    
    for sig_bytes in signatures:
        for pub_key_bytes in council_keys:
            try:
                pub_key = ed25519.Ed25519PublicKey.from_public_bytes(pub_key_bytes)
                pub_key.verify(sig_bytes, registry_bytes)
                valid_count += 1
                break
            except InvalidSignature:
                continue
    
    return valid_count >= threshold

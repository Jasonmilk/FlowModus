import base64
import pytest
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
import flowmodus.control_plane.verifier as verifier_module
from flowmodus.control_plane.verifier import verify_registry, verify_multisig


def _generate_key_pair():
    """Generate a fresh Ed25519 key pair and return (private_key, public_bytes)."""
    private_key = ed25519.Ed25519PrivateKey.generate()
    public_key = private_key.public_key()
    public_bytes = public_key.public_bytes(encoding=Encoding.Raw, format=PublicFormat.Raw)
    return private_key, public_bytes


class TestVerifier:
    """Tests for Ed25519 signature verification."""

    def test_verify_registry_valid(self):
        """Valid signature verification."""
        private_key, pub_bytes = _generate_key_pair()
        data = b"test registry data"
        signature = private_key.sign(data)

        old_key = verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES = pub_bytes
        try:
            assert verify_registry(data, signature) is True
        finally:
            verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES = old_key

    def test_verify_registry_invalid(self):
        """Invalid signature verification (wrong data)."""
        private_key, pub_bytes = _generate_key_pair()
        data = b"test registry data"
        wrong_data = b"wrong data"
        signature = private_key.sign(data)

        old_key = verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES = pub_bytes
        try:
            assert verify_registry(wrong_data, signature) is False
        finally:
            verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES = old_key

    def test_verify_multisig_threshold_met(self):
        """Multisig threshold met: 2 of 3 valid signatures pass."""
        # Generate 3 council key pairs
        private_keys = [ed25519.Ed25519PrivateKey.generate() for _ in range(3)]
        council_keys = [
            pk.public_key().public_bytes(encoding=Encoding.Raw, format=PublicFormat.Raw)
            for pk in private_keys
        ]
        threshold = 2

        # Sign with first two keys
        data = b"test multisig data"
        signatures = []
        for pk in private_keys[:2]:
            signatures.append(pk.sign(data))

        assert verify_multisig(data, signatures, council_keys, threshold) is True

    def test_verify_multisig_threshold_not_met(self):
        """Multisig threshold not met: only 1 of required 3 signatures provided."""
        private_keys = [ed25519.Ed25519PrivateKey.generate() for _ in range(2)]
        council_keys = [
            pk.public_key().public_bytes(encoding=Encoding.Raw, format=PublicFormat.Raw)
            for pk in private_keys
        ]
        threshold = 3  # require 3 signatures, but only 2 keys exist

        data = b"test multisig data"
        signatures = [pk.sign(data) for pk in private_keys]

        assert verify_multisig(data, signatures, council_keys, threshold) is False

    def test_determinism(self):
        """Verification is deterministic."""
        private_key, pub_bytes = _generate_key_pair()
        data = b"test registry data"
        signature = private_key.sign(data)

        old_key = verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES = pub_bytes
        try:
            results = [verify_registry(data, signature) for _ in range(100)]
            assert all(r == results[0] for r in results)
        finally:
            verifier_module.PROTOCOL_ROOT_PUBLIC_KEY_BYTES = old_key

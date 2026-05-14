import pytest
from cryptography.hazmat.primitives.asymmetric import ed25519
from flowmodus.control_plane.verifier import verify_registry, verify_multisig, PROTOCOL_ROOT_PUBLIC_KEY_BYTES


class TestVerifier:

    def test_verify_registry_valid(self):
        """Test valid signature verification"""
        # Generate test key pair
        private_key = ed25519.Ed25519PrivateKey.generate()
        public_key = private_key.public_key()
        
        # Test data
        data = b"test registry data"
        signature = private_key.sign(data)
        
        # Temporarily override the root key for test
        global PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        old_key = PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        PROTOCOL_ROOT_PUBLIC_KEY_BYTES = public_key.public_bytes_raw()
        
        try:
            assert verify_registry(data, signature) is True
        finally:
            PROTOCOL_ROOT_PUBLIC_KEY_BYTES = old_key

    def test_verify_registry_invalid(self):
        """Test invalid signature verification"""
        # Generate test key pair
        private_key = ed25519.Ed25519PrivateKey.generate()
        public_key = private_key.public_key()
        
        # Test data
        data = b"test registry data"
        wrong_data = b"wrong data"
        signature = private_key.sign(data)
        
        # Temporarily override the root key for test
        global PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        old_key = PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        PROTOCOL_ROOT_PUBLIC_KEY_BYTES = public_key.public_bytes_raw()
        
        try:
            assert verify_registry(wrong_data, signature) is False
        finally:
            PROTOCOL_ROOT_PUBLIC_KEY_BYTES = old_key

    def test_verify_multisig_threshold_met(self):
        """Test multisig verification with enough valid signatures"""
        # Generate 3 council keys
        keys = []
        for _ in range(3):
            private = ed25519.Ed25519PrivateKey.generate()
            public = private.public_key()
            keys.append((private, public))
        
        # Test data
        data = b"test registry data"
        signatures = []
        council_keys = []
        
        for private, public in keys:
            signatures.append(private.sign(data))
            council_keys.append(public.public_bytes_raw())
        
        # Verify with threshold 2
        assert verify_multisig(data, signatures, council_keys, 2) is True

    def test_verify_multisig_threshold_not_met(self):
        """Test multisig verification with not enough valid signatures"""
        # Generate 3 council keys
        keys = []
        for _ in range(3):
            private = ed25519.Ed25519PrivateKey.generate()
            public = private.public_key()
            keys.append((private, public))
        
        # Test data
        data = b"test registry data"
        signatures = []
        council_keys = []
        
        # Only 1 valid signature
        private, public = keys[0]
        signatures.append(private.sign(data))
        council_keys.append(public.public_bytes_raw())
        
        # Add wrong signature
        wrong_private = ed25519.Ed25519PrivateKey.generate()
        signatures.append(wrong_private.sign(data))
        
        # Verify with threshold 2
        assert verify_multisig(data, signatures, council_keys, 2) is False

    def test_determinism(self):
        """Verify verification is deterministic"""
        # Generate test key pair
        private_key = ed25519.Ed25519PrivateKey.generate()
        public_key = private_key.public_key()
        
        # Test data
        data = b"test registry data"
        signature = private_key.sign(data)
        
        # Temporarily override the root key for test
        global PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        old_key = PROTOCOL_ROOT_PUBLIC_KEY_BYTES
        PROTOCOL_ROOT_PUBLIC_KEY_BYTES = public_key.public_bytes_raw()
        
        try:
            results = [verify_registry(data, signature) for _ in range(100)]
            assert all(r == results[0] for r in results)
        finally:
            PROTOCOL_ROOT_PUBLIC_KEY_BYTES = old_key

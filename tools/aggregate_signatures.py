#!/usr/bin/env python3
import argparse
import json
import base64
import time
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.exceptions import InvalidSignature


def main():
    parser = argparse.ArgumentParser(description="Aggregate multisig signatures for registry update")
    parser.add_argument("--registry", required=True, help="Path to canonical registry JSON file")
    parser.add_argument("--signatures", required=True, nargs="+", help="Paths to signature JSON files")
    parser.add_argument("--threshold", type=int, required=True, help="Minimum required valid signatures")
    parser.add_argument("--output", required=True, help="Output path for multisig package")
    args = parser.parse_args()

    # Load raw registry bytes for signature verification
    with open(args.registry, "rb") as f:
        registry_bytes = f.read()
    
    # Load registry JSON for package
    with open(args.registry, "r") as f:
        registry_data = json.load(f)

    valid_signatures = []
    for sig_path in args.signatures:
        try:
            with open(sig_path, "r") as f:
                sig_data = json.load(f)
            
            signature = base64.b64decode(sig_data["signature"])
            public_key = base64.b64decode(sig_data["public_key"])

            # Verify signature
            pub_key = ed25519.Ed25519PublicKey.from_public_bytes(public_key)
            pub_key.verify(signature, registry_bytes)
            
            valid_signatures.append(sig_data["signature"])
            print(f"Valid signature from council member: {sig_data.get('member_id', 'unknown')}")
        except Exception as e:
            print(f"Skipping invalid signature from {sig_path}: {str(e)}")
            continue

    if len(valid_signatures) < args.threshold:
        raise ValueError(
            f"Insufficient valid signatures: {len(valid_signatures)} found, {args.threshold} required"
        )

    # Build the final multisig package
    package = {
        "version": registry_data.get("version", "1.0.0"),
        "published_at_unix": int(time.time()),
        "registry": registry_data,
        "signatures": valid_signatures
    }

    with open(args.output, "w") as f:
        json.dump(package, f, indent=2)
    
    print(f"\nSuccess! Aggregated {len(valid_signatures)} valid signatures.")
    print(f"Package written to: {args.output}")


if __name__ == "__main__":
    main()

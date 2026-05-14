#!/bin/bash
# sign_update.sh - Sign and publish registry update to IPNS

set -e

if [ $# -lt 3 ]; then
    echo "Usage: $0 <registry.json> <signatures...> <threshold>"
    exit 1
fi

REGISTRY=$1
shift
THRESHOLD=$1
shift
SIGNATURES=("$@")

python tools/aggregate_signatures.py \
    --registry "$REGISTRY" \
    --signatures "${SIGNATURES[@]}" \
    --threshold "$THRESHOLD" \
    --output multisig_package.json

CID=$(ipfs add -Q multisig_package.json)
ipfs name publish --key=flowmodus $CID

echo "Successfully published registry update to IPNS: $CID"

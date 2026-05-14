import hashlib
from typing import Dict, List

from flowmodus.schemas.routing_pb2 import RawRequest, NormalizedRequest


def calculate_ste(token_count: int, compression_ratio: float) -> float:
    """Calculate Standard Token Equivalent (STE). Pure function."""
    return token_count * compression_ratio


def estimate_token_count(text: str) -> int:
    """
    Estimate token count from raw text using a simple heuristic.
    Pure function, no external tokenizer dependency.
    Approximate; final billing relies on the supplier's actual usage response.
    """
    if not text:
        return 0
    ascii_chars = sum(1 for c in text if ord(c) < 128)
    non_ascii_chars = len(text) - ascii_chars
    return max(1, int(ascii_chars / 4 + non_ascii_chars / 1.5))


def parse_cache_breakpoints(headers: Dict[str, str]) -> List[int]:
    """Extract cache breakpoints from extra headers. Pure function."""
    raw = headers.get("x-flowmodus-cache-breakpoints", "")
    if not raw.strip():
        return []
    try:
        return [int(v.strip()) for v in raw.split(",") if v.strip().isdigit()]
    except ValueError:
        return []


class Layer1Normalizer:
    """Layer 1: Protocol and metric normalization."""

    @staticmethod
    def normalize_request(
        raw: RawRequest,
        tokenizer_ratios: Dict[str, float],
    ) -> NormalizedRequest:
        """Normalize raw user request into standard normalized request. Pure function."""
        prompt_hash = hashlib.sha256(raw.prompt.encode('utf-8')).hexdigest()

        estimated_input_tokens = estimate_token_count(raw.prompt)
        default_ratio = tokenizer_ratios.get("default", 1.0)
        ste_input = calculate_ste(estimated_input_tokens, default_ratio)
        ste_output_estimated = calculate_ste(raw.max_output_tokens, default_ratio)

        cache_breakpoints = parse_cache_breakpoints(dict(raw.extra_headers))

        return NormalizedRequest(
            prompt_hash=prompt_hash,
            estimated_input_tokens=estimated_input_tokens,
            max_output_tokens=raw.max_output_tokens,
            agent_role=raw.agent_role,
            cognitive_mode=raw.cognitive_mode,
            ste_input=ste_input,
            ste_output_estimated=ste_output_estimated,
            cache_breakpoints=cache_breakpoints,
        )

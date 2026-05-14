import hashlib
from typing import Dict

from flowmodus.schemas.routing_pb2 import RawRequest, NormalizedRequest


def calculate_ste(token_count: int, compression_ratio: float) -> float:
    """
    Calculate Standard Token Equivalent (STE).
    Pure function.
    
    Args:
        token_count: Raw token count
        compression_ratio: Tokenizer compression ratio from supplier declaration
    
    Returns:
        STE value
    """
    return token_count * compression_ratio


class Layer1Normalizer:
    """
    Layer 1: Protocol and metric normalization.
    Normalizes raw request into standard format, calculates STE.
    """
    
    @staticmethod
    def normalize_request(
        raw: RawRequest,
        tokenizer_ratios: Dict[str, float],
    ) -> NormalizedRequest:
        """
        Normalize raw user request into standard normalized request.
        Pure function.
        
        Args:
            raw: Raw user request
            tokenizer_ratios: Default tokenizer ratios for estimation
        
        Returns:
            Normalized request
        """
        # Calculate prompt hash for caching
        prompt_hash = hashlib.sha256(raw.prompt.encode('utf-8')).hexdigest()
        
        # Estimate input tokens (rough estimate for now, will be refined)
        # TODO: Use proper tokenizer estimation
        estimated_input_tokens = len(raw.prompt) // 4  # Rough char to token ratio
        
        # Default compression ratio for STE calculation
        default_ratio = tokenizer_ratios.get("default", 1.0)
        ste_input = calculate_ste(estimated_input_tokens, default_ratio)
        ste_output_estimated = calculate_ste(raw.max_output_tokens, default_ratio)
        
        return NormalizedRequest(
            prompt_hash=prompt_hash,
            estimated_input_tokens=estimated_input_tokens,
            max_output_tokens=raw.max_output_tokens,
            agent_role=raw.agent_role,
            cognitive_mode=raw.cognitive_mode,
            ste_input=ste_input,
            ste_output_estimated=ste_output_estimated,
            cache_breakpoints=[]  # TODO: Parse from request
        )

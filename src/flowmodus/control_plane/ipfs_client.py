import httpx
from typing import List


class IpfsClient:
    """
    IPFS/IPNS client for pulling registry.
    Tries multiple gateways sequentially for resilience.
    """
    
    DEFAULT_GATEWAYS = [
        "https://ipfs.io",
        "https://cloudflare-ipfs.com",
        "https://dweb.link",
    ]
    
    def __init__(self, gateways: List[str] = None):
        self._gateways = gateways or self.DEFAULT_GATEWAYS
    
    async def get_ipns_record(self, ipns_name: str) -> bytes:
        """
        Pull IPNS record, trying multiple gateways.
        
        Args:
            ipns_name: IPNS name to resolve
        
        Returns:
            Raw registry bytes
        """
        for gateway in self._gateways:
            try:
                async with httpx.AsyncClient(timeout=10.0) as client:
                    response = await client.get(
                        f"{gateway}/ipns/{ipns_name}",
                        follow_redirects=True
                    )
                    response.raise_for_status()
                    return response.content
            except Exception:
                # Try next gateway
                continue
        
        raise RuntimeError("Failed to fetch IPNS record from all gateways")

import asyncio
import time
import httpx
from aiohttp import web
from typing import Optional
from httpx import ConnectTimeout, ConnectError

from flowmodus.lifecycle import Sidecar, SidecarState


def classify_http_error(status_code: int, exception: Optional[Exception] = None) -> str:
    """
    Classify HTTP error into retryable/terminal.
    Pure function.
    
    Args:
        status_code: HTTP status code
        exception: Exception if any
    
    Returns:
        Error classification: "success", "retryable", "terminal"
    """
    if 200 <= status_code < 300:
        return "success"
    
    if status_code in (429, 500, 502, 503, 504):
        return "retryable"
    
    if isinstance(exception, (ConnectTimeout, ConnectError)):
        return "retryable"
    
    if status_code in (401, 403, 400):
        return "terminal"
    
    return "retryable"


class Proxy:
    """
    HTTP proxy for LLM requests.
    Handles incoming requests, runs routing pipeline, proxies to selected supplier.
    """
    
    def __init__(self, sidecar: Sidecar):
        self._sidecar = sidecar
        self._app = web.Application()
        self._app.add_routes([
            web.post('/v1/chat/completions', self._handle_chat_completion),
        ])
        self._runner: Optional[web.AppRunner] = None
    
    async def start(self, host: str = "127.0.0.1", port: int = 8080) -> None:
        """Start the proxy server."""
        self._runner = web.AppRunner(self._app)
        await self._runner.setup()
        site = web.TCPSite(self._runner, host, port)
        await site.start()
    
    async def stop(self) -> None:
        """Stop the proxy server."""
        if self._runner:
            await self._runner.cleanup()
    
    async def _handle_chat_completion(self, request: web.Request) -> web.Response:
        """Handle chat completion request."""
        if self._sidecar.state != SidecarState.READY:
            return web.json_response(
                {"error": "Sidecar not ready"},
                status=503
            )
        
        # Track inflight request
        self._sidecar.track_inflight_request()
        try:
            # Parse raw request
            body = await request.json()
            raw_request = RawRequest(
                prompt=body.get("messages", [{}])[-1].get("content", ""),
                agent_role=body.get("agent_role", "default"),
                cognitive_mode=body.get("cognitive_mode", "default"),
                max_output_tokens=body.get("max_tokens", 1024),
            )
            
            # Run routing pipeline
            decision = await self._run_pipeline(raw_request)
            
            # Proxy the request
            return await self._proxy_request(request, decision, body)
        finally:
            self._sidecar.untrack_inflight_request()
    
    async def _run_pipeline(self, raw_request: RawRequest) -> RoutingDecision:
        """Run the 5-layer routing pipeline."""
        # Layer 1: Normalize
        normalized = Layer1Normalizer.normalize_request(
            raw_request,
            {"default": 1.0}  # TODO: Default tokenizer ratios
        )
        
        # Layer 2: Registry lookup
        layer2 = Layer2Registry(self._sidecar._registry_snapshot)
        suppliers = layer2.get_eligible_suppliers(normalized)
        
        # Layer 3: Cost estimation
        layer3 = Layer3CostEstimator(self._sidecar._deviation_snapshot)
        estimates = layer3.estimate_all(normalized, suppliers)
        
        # Layer 4: Hard filters
        # TODO: Load user constraints and bias config
        constraints = ...
        bias_config = ...
        health_states = ...
        layer4 = Layer4HardFilter(
            constraints,
            self._sidecar._deviation_snapshot,
            bias_config,
            health_states,
            self._sidecar.instance_id
        )
        filtered = layer4.filter(estimates)
        
        # Convert to eligible suppliers
        eligible = []
        for estimate in filtered:
            # TODO: Get endpoint URL
            eligible.append(...)
        
        # Layer 5: Score and select
        layer5 = Layer5Scorer(
            normalized.agent_role,
            self._sidecar.instance_id,
            normalized.prompt_hash,
            bias_config
        )
        decision = layer5.select(eligible)
        
        return decision
    
    async def _proxy_request(
        self,
        original_request: web.Request,
        decision: RoutingDecision,
        body: dict,
    ) -> web.Response:
        """Proxy the request to the selected supplier."""
        start_time = time.monotonic()
        
        async with httpx.AsyncClient() as client:
            # Modify request for supplier
            # TODO: Translate cache breakpoints
            response = await client.post(
                decision.endpoint_url,
                json=body,
                headers=dict(original_request.headers),
                stream=True
            )
            
            # Wrap stream to measure TTFT
            timed_stream = TTFTTimedStream(response.aiter_bytes(), start_time)
            
            # Create response
            aiohttp_response = web.StreamResponse(
                status=response.status_code,
                headers=dict(response.headers)
            )
            await aiohttp_response.prepare(original_request)
            
            # Stream the response
            async for chunk in timed_stream:
                await aiohttp_response.write(chunk)
            
            # Collect telemetry
            total_duration = (time.monotonic() - start_time) * 1000
            # TODO: Parse usage from response
            usage = {}
            self._sidecar.telemetry_collector.collect_from_response(
                supplier_id=decision.supplier_id,
                model_id=decision.model_id,
                status_code=response.status_code,
                ttft_ms=timed_stream.ttft_ms or 0.0,
                total_duration_ms=total_duration,
                response_headers=dict(response.headers),
                usage=usage
            )
            
            return aiohttp_response

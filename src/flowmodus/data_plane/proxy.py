import time
from typing import Optional
import httpx
from aiohttp import web

from flowmodus.schemas.routing_pb2 import RawRequest, RoutingDecision
from flowmodus.data_plane.http_utils import classify_http_error
from flowmodus.data_plane.pipeline import (
    Layer1Normalizer,
    Layer2Registry,
    Layer3CostEstimator,
    Layer4HardFilter,
    Layer5Scorer,
)
from flowmodus.data_plane.telemetry import TelemetryCollector, TTFTTimedStream
from flowmodus.config.bias import BiasConfig
from flowmodus.lifecycle import Sidecar, SidecarState


class Proxy:
    """HTTP proxy for LLM requests. Handles incoming requests, runs routing pipeline, proxies to selected supplier."""

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
                status=503,
            )

        self._sidecar.track_inflight_request()
        try:
            body = await request.json()
            raw_request = RawRequest(
                prompt=body.get("messages", [{}])[-1].get("content", ""),
                agent_role=body.get("agent_role", "default"),
                cognitive_mode=body.get("cognitive_mode", "default"),
                max_output_tokens=body.get("max_tokens", 1024),
            )

            decision = await self._run_pipeline(raw_request)
            return await self._proxy_request(request, decision, body)
        finally:
            self._sidecar.untrack_inflight_request()

    async def _run_pipeline(self, raw_request: RawRequest) -> RoutingDecision:
        """Run the 5-layer routing pipeline."""
        # Layer 1: Normalize
        normalized = Layer1Normalizer.normalize_request(
            raw_request,
            self._sidecar.tokenizer_ratios,
        )

        # Layer 2: Registry lookup
        layer2 = Layer2Registry(self._sidecar.registry_snapshot)
        suppliers = layer2.get_eligible_suppliers(normalized)

        # Layer 3: Cost estimation
        layer3 = Layer3CostEstimator(self._sidecar.deviation_snapshot)
        estimates = layer3.estimate_all(normalized, suppliers)

        # Layer 4: Hard filters
        layer4 = Layer4HardFilter(
            self._sidecar.user_constraints,
            self._sidecar.deviation_snapshot,
            self._sidecar.bias_config,
            self._sidecar.health_states,
            self._sidecar.health_timestamps,
            self._sidecar.instance_id,
            self._sidecar.current_minute,
        )
        filtered = layer4.filter(estimates)

        # Convert to eligible suppliers
        eligible = []
        for estimate in filtered:
            endpoint_url = self._sidecar.get_endpoint_url(
                estimate.supplier_id, estimate.model_id
            )
            if endpoint_url:
                eligible.append(
                    self._sidecar.create_eligible_supplier(estimate, endpoint_url)
                )

        # Layer 5: Score and select
        layer5 = Layer5Scorer(
            normalized.agent_role,
            self._sidecar.instance_id,
            normalized.prompt_hash,
            self._sidecar.bias_config,
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

        # Translate cache breakpoints into the selected supplier's format
        adapted_body = self._sidecar.translate_cache_breakpoints(body, decision.supplier_id)

        async with httpx.AsyncClient() as client:
            response = await client.post(
                decision.endpoint_url,
                json=adapted_body,
                headers=dict(original_request.headers),
                stream=True,
            )

            timed_stream = TTFTTimedStream(response.aiter_bytes(), start_time)
            aiohttp_response = web.StreamResponse(
                status=response.status_code,
                headers=dict(response.headers),
            )
            await aiohttp_response.prepare(original_request)

            async for chunk in timed_stream:
                await aiohttp_response.write(chunk)

            total_duration = (time.monotonic() - start_time) * 1000
            # Collect telemetry (actual usage parsing delegated to Sidecar)
            usage = self._sidecar.extract_usage_from_response(response)

            self._sidecar.telemetry_collector.collect_from_response(
                supplier_id=decision.supplier_id,
                model_id=decision.model_id,
                status_code=response.status_code,
                ttft_ms=timed_stream.ttft_ms or 0.0,
                total_duration_ms=total_duration,
                response_headers=dict(response.headers),
                usage=usage,
            )

            return aiohttp_response

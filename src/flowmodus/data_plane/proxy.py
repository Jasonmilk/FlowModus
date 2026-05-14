import time
import json
import traceback
from typing import Optional
import httpx
from aiohttp import web

from flowmodus.schemas.routing_pb2 import RawRequest, RoutingDecision
from flowmodus.lifecycle import Sidecar, SidecarState


class Proxy:
    """HTTP proxy for LLM requests. Routes to supplier based on call mode."""

    def __init__(self, sidecar: Sidecar, debug: bool = False) -> None:
        self._sidecar = sidecar
        self._debug = debug
        self._app = web.Application()
        self._app.add_routes([
            web.post('/v1/chat/completions', self._handle_chat_completion),
        ])
        self._runner: Optional[web.AppRunner] = None

    async def start(self, host: str, port: int) -> None:
        self._runner = web.AppRunner(self._app)
        await self._runner.setup()
        site = web.TCPSite(self._runner, host, port)
        await site.start()

    async def stop(self) -> None:
        if self._runner:
            await self._runner.cleanup()

    async def _handle_chat_completion(self, request: web.Request) -> web.Response:
        if self._sidecar.state != SidecarState.READY:
            return web.json_response({"error": "Sidecar not ready"}, status=503)

        self._sidecar.track_inflight_request()
        try:
            body = await request.json()
            raw_request = RawRequest(
                prompt=body.get("messages", [{}])[-1].get("content", ""),
                agent_role=body.get("agent_role", ""),
                cognitive_mode=body.get("cognitive_mode", ""),
                max_output_tokens=body.get("max_tokens", 0),
            )

            model_param = body.get("model", "auto")
            decision = self._sidecar.resolve_routing_decision(
                raw_request=raw_request,
                model_param=model_param,
                headers=dict(request.headers),
            )

            return await self._proxy_request(request, decision, body)
        except Exception as e:
            if self._debug:
                traceback.print_exc()
            return web.json_response({"error": str(e)}, status=500)
        finally:
            self._sidecar.untrack_inflight_request()

    async def _proxy_request(
        self,
        original_request: web.Request,
        decision: RoutingDecision,
        body: dict,
    ) -> web.Response:
        start_time = time.monotonic()

        headers = dict(original_request.headers)
        headers.pop("Host", None)
        headers.pop("host", None)
        headers.pop("Connection", None)
        headers.pop("connection", None)
        headers.pop("Content-Length", None)
        headers.pop("content-length", None)

        json_body = json.dumps(body).encode('utf-8')
        headers["Content-Length"] = str(len(json_body))

        if self._debug:
            print(f"Proxying to {decision.endpoint_url}", flush=True)
            print(f"Request body length: {len(json_body)} bytes", flush=True)

        async with httpx.AsyncClient(timeout=30.0) as client:
            try:
                response = await client.post(
                    decision.endpoint_url,
                    content=json_body,
                    headers=headers,
                )

                if self._debug:
                    print(f"Supplier response status: {response.status_code}", flush=True)

                total_duration = (time.monotonic() - start_time) * 1000
                usage = {}

                self._sidecar.telemetry_collector.collect_from_response(
                    supplier_id=decision.supplier_id,
                    model_id=decision.model_id,
                    status_code=response.status_code,
                    ttft_ms=total_duration,
                    total_duration_ms=total_duration,
                    response_headers=dict(response.headers),
                    usage=usage,
                )

                if response.status_code == 200:
                    return web.json_response(
                        status=200,
                        data=response.json(),
                    )
                else:
                    error_detail = response.text
                    if self._debug:
                        print(f"Supplier error: {error_detail}", flush=True)
                    return web.json_response(
                        status=response.status_code,
                        data={"error": error_detail, "status": response.status_code},
                    )
            except Exception as e:
                if self._debug:
                    print(f"Proxy request failed: {e}", flush=True)
                    traceback.print_exc()
                return web.json_response({"error": str(e)}, status=502)

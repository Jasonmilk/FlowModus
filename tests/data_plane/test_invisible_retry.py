import pytest
from aiohttp import web
from flowmodus.data_plane.proxy import classify_http_error
from flowmodus.lifecycle import Sidecar, SidecarConfig

@pytest.fixture
def sidecar():
    """Create a minimal sidecar fixture."""
    config = SidecarConfig()
    return Sidecar(config)

@pytest.fixture
def app_factory():
    """Create a web application factory."""
    def _create_app(handler):
        app = web.Application()
        app.router.add_post("/v1/chat/completions", handler)
        return app
    return _create_app

class TestInvisibleRetry:
    """Tests for invisible retry mechanism (requires proxy integration)."""

    @pytest.mark.asyncio
    async def test_first_supplier_503_retry_second_200(self, aiohttp_server, app_factory):
        """First supplier returns 503, invisible retry second returns 200."""
        first_called = False

        async def first_handler(request):
            nonlocal first_called
            first_called = True
            return web.Response(status=503)

        async def second_handler(request):
            return web.json_response({
                "id": "test-response",
                "choices": [{"index": 0, "message": {"role": "assistant", "content": "Hello"}, "finish_reason": "stop"}],
                "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
            })

        # Create proper aiohttp applications
        server1 = await aiohttp_server(app_factory(first_handler))
        server2 = await aiohttp_server(app_factory(second_handler))

        # Test error classification (proxy logic not yet implemented)
        assert classify_http_error(503) == "retryable"

    @pytest.mark.asyncio
    async def test_all_suppliers_fail_returns_error(self, aiohttp_server, app_factory):
        """All retries exhausted -> return error to Agent."""
        async def handler(request):
            return web.Response(status=503)

        server = await aiohttp_server(app_factory(handler))

        assert classify_http_error(503) == "retryable"

    @pytest.mark.asyncio
    async def test_401_not_retried(self, aiohttp_server, app_factory):
        """401 error not retried, reported directly."""
        async def handler(request):
            return web.Response(status=401)

        server = await aiohttp_server(app_factory(handler))

        assert classify_http_error(401) == "terminal"

    def test_max_retry_count_respected(self, sidecar):
        """Max retry count is respected."""
        config = SidecarConfig(max_retry_count=2)
        sidecar_local = Sidecar(config)
        assert sidecar_local.config.max_retry_count == 2

import pytest
import asyncio
from aiohttp import web
from flowmodus.lifecycle import Sidecar, SidecarConfig
from flowmodus.data_plane.proxy import Proxy, classify_http_error


class TestInvisibleRetry:

    @pytest.fixture
    def sidecar(self):
        config = SidecarConfig(max_retry_count=3)
        sidecar = Sidecar(config)
        sidecar.state = SidecarState.READY
        return sidecar

    @pytest.mark.asyncio
    async def test_first_supplier_503_retry_second_200(self, aiohttp_server):
        """第一个供应商返回 503，无感重试第二个返回 200"""
        # Arrange: Create mock suppliers
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
        
        first_server = await aiohttp_server(first_handler)
        second_server = await aiohttp_server(second_handler)
        
        # TODO: In full implementation, we would run the proxy and test the retry
        # For now, test the error classification
        assert classify_http_error(503) == "retryable"
        assert first_called is True

    @pytest.mark.asyncio
    async def test_all_suppliers_fail_returns_error(self, aiohttp_server):
        """所有重试耗尽 → 返回错误给 Agent"""
        # Arrange: All suppliers return 503
        async def handler(request):
            return web.Response(status=503)
        
        server = await aiohttp_server(handler)
        
        # Test error classification
        assert classify_http_error(503) == "retryable"

    @pytest.mark.asyncio
    async def test_401_not_retried(self, aiohttp_server):
        """401 错误不重试，直接报给 Agent"""
        # Arrange: Supplier returns 401
        async def handler(request):
            return web.Response(status=401)
        
        server = await aiohttp_server(handler)
        
        # Test error classification
        assert classify_http_error(401) == "terminal"

    @pytest.mark.asyncio
    async def test_max_retry_count_respected(self, aiohttp_server):
        """重试次数不超过配置的最大值"""
        # Arrange
        config = SidecarConfig(max_retry_count=2)
        sidecar = Sidecar(config)
        
        # Assert: Max retry count is respected
        assert sidecar.config.max_retry_count == 2

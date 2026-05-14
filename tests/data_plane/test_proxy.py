import pytest
from flowmodus.data_plane.proxy import classify_http_error


class TestClassifyHttpError:

    # --- success ---
    def test_2xx_is_success(self):
        for code in [200, 201, 204]:
            assert classify_http_error(code) == "success"

    # --- retryable ---
    def test_429_is_retryable(self):
        assert classify_http_error(429) == "retryable"

    def test_5xx_is_retryable(self):
        for code in [500, 502, 503, 504]:
            assert classify_http_error(code) == "retryable"

    # --- terminal ---
    def test_401_is_terminal(self):
        assert classify_http_error(401) == "terminal"

    def test_403_is_terminal(self):
        assert classify_http_error(403) == "terminal"

    def test_400_is_terminal(self):
        assert classify_http_error(400) == "terminal"

    # --- unknown ---
    def test_unknown_defaults_to_retryable(self):
        """未知状态码默认可重试一次"""
        assert classify_http_error(418) == "retryable"

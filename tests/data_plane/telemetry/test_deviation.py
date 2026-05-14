import pytest
from flowmodus.data_plane.telemetry.deviation import calculate_deviation


class TestCalculateDeviation:

    def test_exact_match(self):
        """声明与实测一致"""
        assert calculate_deviation(100.0, 100.0) == 0.0

    def test_actual_higher(self):
        """实测大于声明（更差）"""
        deviation = calculate_deviation(100.0, 150.0)
        assert deviation == 50.0

    def test_actual_lower(self):
        """实测小于声明（更好）"""
        deviation = calculate_deviation(100.0, 80.0)
        assert deviation == -20.0

    def test_claimed_zero_actual_zero(self):
        """声明和实测均为零"""
        assert calculate_deviation(0.0, 0.0) == 0.0

    def test_claimed_zero_actual_nonzero(self):
        """声明免费但实测有成本"""
        deviation = calculate_deviation(0.0, 0.05)
        assert deviation == 5.0

    def test_large_deviation(self):
        """极端偏差"""
        deviation = calculate_deviation(1.0, 100.0)
        assert deviation == 9900.0

    def test_determinism(self):
        results = [calculate_deviation(3.14, 2.71) for _ in range(100)]
        assert all(r == results[0] for r in results)

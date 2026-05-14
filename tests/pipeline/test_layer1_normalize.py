import pytest
from flowmodus.data_plane.pipeline.layer1_normalize import calculate_ste


class TestCalculateSTE:
    """STE = token_count * compression_ratio"""

    def test_baseline_ratio(self):
        """基准压缩率为 1.0，STE 等于原始 Token 数"""
        assert calculate_ste(100, 1.0) == 100.0

    def test_high_compression(self):
        """高压缩率意味着相同 Token 包含更多信息"""
        assert calculate_ste(100, 1.5) == 150.0

    def test_low_compression(self):
        """低压缩率"""
        assert calculate_ste(100, 0.5) == 50.0

    def test_zero_tokens(self):
        """零 Token"""
        assert calculate_ste(0, 1.0) == 0.0
        assert calculate_ste(0, 2.5) == 0.0

    def test_zero_ratio(self):
        """压缩率为零（理论上不应该发生，但边界要覆盖）"""
        assert calculate_ste(100, 0.0) == 0.0

    def test_negative_ratio(self):
        """负压缩率（物理上不可能，但函数签名不限制）"""
        assert calculate_ste(100, -1.0) == -100.0

    def test_determinism(self):
        """确定性验证：相同输入 100 次必须产生相同输出"""
        results = [calculate_ste(42, 1.337) for _ in range(100)]
        assert all(r == results[0] for r in results)
        assert results[0] == 42 * 1.337

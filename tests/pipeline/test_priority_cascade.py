import pytest
from flowmodus.data_plane.pipeline.layer4_filter import apply_priority_filter
from flowmodus.schemas.routing_pb2 import CostEstimate
from flowmodus.config.bias import BiasConfig, PriorityGroup


def _make_config():
    return BiasConfig()


class TestPriorityCascade:

    def test_healthy_fast_lane_used(self):
        """fast-lane has HEALTHY supplier -> only fast-lane returned."""
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
        ]
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o"]),
        ]
        health_states = {"fast1": "HEALTHY", "fast2": "HEALTHY", "backup1": "HEALTHY"}
        health_timestamps = {}
        instance_id = "test-instance"
        current_minute = 42
        bias_config = _make_config()
        bias_config.priority_groups = priority_groups

        result = apply_priority_filter(
            candidates, priority_groups, health_states, health_timestamps,
            instance_id, current_minute, bias_config,
        )

        assert len(result) == 2
        supplier_ids = {c.supplier_id for c in result}
        assert supplier_ids == {"fast1", "fast2"}

    def test_degraded_excluded_without_rehab(self):
        """DEGRADED without rehab -> excluded."""
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
        ]
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o"]),
        ]
        health_states = {"fast1": "DEGRADED", "fast2": "HEALTHY", "backup1": "HEALTHY"}
        health_timestamps = {"fast1": 0}
        instance_id = "test-instance"
        current_minute = 42
        bias_config = _make_config()
        bias_config.priority_groups = priority_groups

        result = apply_priority_filter(
            candidates, priority_groups, health_states, health_timestamps,
            instance_id, current_minute, bias_config,
        )

        assert len(result) == 1
        assert result[0].supplier_id == "fast2"

    def test_terminal_excluded(self):
        """TERMINAL -> excluded."""
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
        ]
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o"]),
        ]
        health_states = {"fast1": "TERMINAL", "fast2": "TERMINAL", "backup1": "HEALTHY"}
        health_timestamps = {}
        instance_id = "test-instance"
        current_minute = 42
        bias_config = _make_config()
        bias_config.priority_groups = priority_groups

        result = apply_priority_filter(
            candidates, priority_groups, health_states, health_timestamps,
            instance_id, current_minute, bias_config,
        )

        assert len(result) == 1
        assert result[0].supplier_id == "backup1"

    def test_falls_back_when_fast_lane_empty(self):
        """fast-lane empty -> falls back to backup-lane."""
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
        ]
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o"]),
        ]
        health_states = {"fast1": "TERMINAL", "fast2": "TERMINAL", "backup1": "HEALTHY"}
        health_timestamps = {}
        instance_id = "test-instance"
        current_minute = 42
        bias_config = _make_config()
        bias_config.priority_groups = priority_groups

        result = apply_priority_filter(
            candidates, priority_groups, health_states, health_timestamps,
            instance_id, current_minute, bias_config,
        )

        assert len(result) == 1
        assert result[0].supplier_id == "backup1"

    def test_all_groups_empty_returns_all(self):
        """All groups empty -> returns all candidates."""
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
            CostEstimate(supplier_id="other1", model_id="other/model", estimated_cost_usd=0.0001),
        ]
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o"]),
        ]
        health_states = {
            "fast1": "TERMINAL", "fast2": "TERMINAL",
            "backup1": "TERMINAL", "other1": "HEALTHY",
        }
        health_timestamps = {}
        instance_id = "test-instance"
        current_minute = 42
        bias_config = _make_config()
        bias_config.priority_groups = priority_groups

        result = apply_priority_filter(
            candidates, priority_groups, health_states, health_timestamps,
            instance_id, current_minute, bias_config,
        )

        assert len(result) == 4
        supplier_ids = {c.supplier_id for c in result}
        assert "other1" in supplier_ids

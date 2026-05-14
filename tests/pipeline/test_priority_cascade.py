import pytest
from flowmodus.schemas.routing_pb2 import CostEstimate
from flowmodus.data_plane.pipeline.layer4_filter import apply_priority_filter
from flowmodus.config.bias import PriorityGroup


class TestPriorityCascade:

    def test_healthy_fast_lane_used(self):
        """fast-lane 有 HEALTHY 供应商 → 只返回 fast-lane"""
        # Arrange
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
        ]
        
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o"]),
        ]
        
        health_states = {
            "fast1": "HEALTHY",
            "fast2": "HEALTHY",
            "backup1": "HEALTHY",
        }
        
        instance_id = "test-instance"
        
        # Act
        result = apply_priority_filter(candidates, priority_groups, health_states, instance_id)
        
        # Assert: Only fast lane candidates are returned
        assert len(result) == 2
        assert {c.supplier_id for c in result} == {"fast1", "fast2"}

    def test_degraded_excluded_without_rehab(self):
        """DEGRADED 且不满足复健条件 → 排除"""
        # Arrange
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
        ]
        
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o"]),
        ]
        
        health_states = {
            "fast1": "DEGRADED",  # Degraded, not rehab time
            "fast2": "HEALTHY",
            "backup1": "HEALTHY",
        }
        
        instance_id = "test-instance"
        
        # Act
        result = apply_priority_filter(candidates, priority_groups, health_states, instance_id)
        
        # Assert: fast1 is excluded, only fast2 remains
        assert len(result) == 1
        assert result[0].supplier_id == "fast2"

    def test_terminal_excluded(self):
        """TERMINAL 状态 → 排除"""
        # Arrange
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
        ]
        
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o"]),
        ]
        
        health_states = {
            "fast1": "TERMINAL",  # Terminal, excluded
            "fast2": "TERMINAL",  # Terminal, excluded
            "backup1": "HEALTHY",
        }
        
        instance_id = "test-instance"
        
        # Act
        result = apply_priority_filter(candidates, priority_groups, health_states, instance_id)
        
        # Assert: All fast lane are excluded, fall back to backup
        assert len(result) == 1
        assert result[0].supplier_id == "backup1"

    def test_falls_back_when_fast_lane_empty(self):
        """fast-lane 全部不可用 → 穿透到 backup-lane"""
        # Arrange
        candidates = [
            CostEstimate(supplier_id="fast1", model_id="openai/gpt-4o", estimated_cost_usd=0.01),
            CostEstimate(supplier_id="fast2", model_id="deepseek/v3", estimated_cost_usd=0.005),
            CostEstimate(supplier_id="backup1", model_id="cheap/gpt-4o", estimated_cost_usd=0.001),
            CostEstimate(supplier_id="backup2", model_id="cheap/gpt-3.5", estimated_cost_usd=0.0005),
        ]
        
        priority_groups = [
            PriorityGroup(name="fast-lane", priority=10, models=["openai/gpt-4o", "deepseek/v3"]),
            PriorityGroup(name="backup-lane", priority=1, models=["cheap/gpt-4o", "cheap/gpt-3.5"]),
        ]
        
        health_states = {
            "fast1": "TERMINAL",
            "fast2": "TERMINAL",
            "backup1": "HEALTHY",
            "backup2": "HEALTHY",
        }
        
        instance_id = "test-instance"
        
        # Act
        result = apply_priority_filter(candidates, priority_groups, health_states, instance_id)
        
        # Assert: Fall back to backup lane
        assert len(result) == 2
        assert {c.supplier_id for c in result} == {"backup1", "backup2"}

    def test_all_groups_empty_returns_all(self):
        """所有组都不可用 → 返回全部候选人（让上层决定）"""
        # Arrange
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
            "fast1": "TERMINAL",
            "fast2": "TERMINAL",
            "backup1": "TERMINAL",
            "other1": "HEALTHY",
        }
        
        instance_id = "test-instance"
        
        # Act
        result = apply_priority_filter(candidates, priority_groups, health_states, instance_id)
        
        # Assert: Return all candidates, since all groups are empty
        assert len(result) == 1
        assert result[0].supplier_id == "other1"

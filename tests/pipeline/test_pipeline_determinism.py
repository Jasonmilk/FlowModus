import pytest
from flowmodus.schemas.routing_pb2 import CostEstimate, UserConstraints
from flowmodus.schemas.routing_pb2 import EligibleSupplier
from flowmodus.data_plane.pipeline.layer4_filter import apply_hard_filters
from flowmodus.data_plane.pipeline.layer5_score import score_and_entropy_sample
from flowmodus.data_plane.telemetry import DeviationSnapshot
from flowmodus.config.bias import BiasConfig, PriorityGroup


class TestPipelineDeterminism:

    def test_apply_hard_filters_determinism(self):
        """Layer 4 确定性验证"""
        # Arrange: Create fixed test input
        candidates = [
            CostEstimate(
                supplier_id="test1",
                model_id="model1",
                estimated_cost_usd=0.01,
                ste_total=100.0,
                kv_cache_applicable=True
            ),
            CostEstimate(
                supplier_id="test2",
                model_id="model2",
                estimated_cost_usd=0.02,
                ste_total=200.0,
                kv_cache_applicable=False
            ),
            CostEstimate(
                supplier_id="test3",
                model_id="model3",
                estimated_cost_usd=0.005,
                ste_total=50.0,
                kv_cache_applicable=True
            )
        ]
        
        constraints = UserConstraints(
            max_cost_per_request_usd=0.015,
            require_verified_supplier=False,
            max_claim_deviation_tolerance=10.0
        )
        
        deviation_snapshot = DeviationSnapshot({})
        bias_config = BiasConfig()
        health_states = {}
        instance_id = "test-instance-123"
        
        # Act: Call 100 times
        results = []
        for _ in range(100):
            result = apply_hard_filters(
                candidates,
                constraints,
                deviation_snapshot,
                bias_config,
                health_states,
                instance_id
            )
            results.append(result)
        
        # Assert: All results are identical
        first = results[0]
        for res in results[1:]:
            # Compare each candidate's fields
            assert len(res) == len(first)
            for a, b in zip(res, first):
                assert a.supplier_id == b.supplier_id
                assert a.estimated_cost_usd == b.estimated_cost_usd

    def test_entropy_sampling_determinism_same_instance(self):
        """相同 instance_id + 相同请求 → 相同采样结果"""
        # Arrange: Create fixed test input
        candidates = [
            EligibleSupplier(
                supplier_id="test1",
                model_id="model1",
                endpoint_url="http://test1.com",
                score=100.0,
                deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.01)
            ),
            EligibleSupplier(
                supplier_id="test2",
                model_id="model2",
                endpoint_url="http://test2.com",
                score=90.0,
                deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.005)
            ),
            EligibleSupplier(
                supplier_id="test3",
                model_id="model3",
                endpoint_url="http://test3.com",
                score=80.0,
                deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.001)
            )
        ]
        
        agent_role = "default"
        instance_id = "test-instance-123"
        request_hash = "test-request-hash-456"
        bias_config = BiasConfig()
        
        # Act: Call 100 times
        results = []
        for _ in range(100):
            decision = score_and_entropy_sample(
                candidates,
                agent_role,
                instance_id,
                request_hash,
                bias_config
            )
            results.append(decision)
        
        # Assert: All decisions are identical
        first = results[0]
        for res in results[1:]:
            assert res.supplier_id == first.supplier_id
            assert res.model_id == first.model_id
            assert res.estimated_cost_usd == first.estimated_cost_usd

    def test_entropy_sampling_divergence_different_instance(self):
        """不同 instance_id + 相同请求 → 不同采样结果"""
        # Arrange: Create fixed test input
        candidates = [
            EligibleSupplier(
                supplier_id="test1",
                model_id="model1",
                endpoint_url="http://test1.com",
                score=100.0,
                deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.01)
            ),
            EligibleSupplier(
                supplier_id="test2",
                model_id="model2",
                endpoint_url="http://test2.com",
                score=90.0,
                deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.005)
            ),
            EligibleSupplier(
                supplier_id="test3",
                model_id="model3",
                endpoint_url="http://test3.com",
                score=80.0,
                deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.001)
            )
        ]
        
        agent_role = "default"
        request_hash = "test-request-hash-456"
        bias_config = BiasConfig()
        
        # Act: Call with different instance IDs
        decisions = []
        for i in range(10):
            instance_id = f"test-instance-{i}"
            decision = score_and_entropy_sample(
                candidates,
                agent_role,
                instance_id,
                request_hash,
                bias_config
            )
            decisions.append(decision)
        
        # Assert: We get different decisions (global de-correlation)
        # Note: Not 100% guaranteed, but with 10 different instances,
        # we should see at least some divergence
        unique_suppliers = set(d.supplier_id for d in decisions)
        assert len(unique_suppliers) > 1

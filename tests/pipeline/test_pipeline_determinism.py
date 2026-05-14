import pytest
from flowmodus.schemas.routing_pb2 import CostEstimate, UserConstraints, EligibleSupplier
from flowmodus.data_plane.telemetry import DeviationSnapshot
from flowmodus.data_plane.pipeline.layer4_filter import apply_hard_filters
from flowmodus.data_plane.pipeline.layer5_score import score_and_entropy_sample
from flowmodus.config.bias import BiasConfig


class TestPipelineDeterminism:

    def test_apply_hard_filters_determinism(self):
        """Layer 4 determinism: same input 100 times -> same output."""
        candidates = [
            CostEstimate(
                supplier_id="test1",
                model_id="model1",
                estimated_cost_usd=0.01,
                ste_total=100.0,
                kv_cache_applicable=True,
            ),
            CostEstimate(
                supplier_id="test2",
                model_id="model2",
                estimated_cost_usd=0.02,
                ste_total=200.0,
                kv_cache_applicable=False,
            ),
            CostEstimate(
                supplier_id="test3",
                model_id="model3",
                estimated_cost_usd=0.005,
                ste_total=50.0,
                kv_cache_applicable=True,
            ),
        ]

        constraints = UserConstraints(
            max_cost_per_request_usd=0.015,
            require_verified_supplier=False,
            max_claim_deviation_tolerance=10.0,
        )

        deviation_snapshot = DeviationSnapshot({})
        bias_config = BiasConfig()
        health_states = {}
        health_timestamps = {}
        instance_id = "test-instance-123"
        current_minute = 42

        results = []
        for _ in range(100):
            result = apply_hard_filters(
                candidates,
                constraints,
                deviation_snapshot,
                bias_config,
                health_states,
                health_timestamps,
                instance_id,
                current_minute,
            )
            results.append([c.supplier_id for c in result])

        assert all(r == results[0] for r in results)

    def test_entropy_sampling_determinism_same_instance(self):
        """Same instance_id + same request -> same sampling result."""
        candidates = [
            EligibleSupplier(
                supplier_id="test1",
                model_id="model1",
                endpoint_url="http://test1.com",
                score=100.0,
                claim_deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.01),
            ),
            EligibleSupplier(
                supplier_id="test2",
                model_id="model2",
                endpoint_url="http://test2.com",
                score=90.0,
                claim_deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.005),
            ),
            EligibleSupplier(
                supplier_id="test3",
                model_id="model3",
                endpoint_url="http://test3.com",
                score=80.0,
                claim_deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.001),
            ),
        ]

        agent_role = "default"
        request_hash = "test-request-hash-123"
        bias_config = BiasConfig()
        instance_id = "fixed-instance"

        decisions = []
        for _ in range(100):
            decision = score_and_entropy_sample(
                candidates,
                agent_role,
                instance_id,
                request_hash,
                bias_config,
            )
            decisions.append(decision.supplier_id)

        assert all(d == decisions[0] for d in decisions)

    
    def test_entropy_sampling_divergence_different_instance(self):
        """Different instance_id + same request -> different sampling results."""
        candidates = [
            EligibleSupplier(
                supplier_id="test1",
                model_id="model1",
                endpoint_url="http://test1.com",
                score=100.0,
                claim_deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.01),
            ),
            EligibleSupplier(
                supplier_id="test2",
                model_id="model2",
                endpoint_url="http://test2.com",
                score=99.0,
                claim_deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.005),
            ),
            EligibleSupplier(
                supplier_id="test3",
                model_id="model3",
                endpoint_url="http://test3.com",
                score=98.0,
                claim_deviation=0.0,
                cost=CostEstimate(estimated_cost_usd=0.001),
            ),
        ]

        agent_role = "default"
        request_hash = "test-request-hash-456"
        bias_config = BiasConfig()

        decisions = []
        for i in range(100):
            instance_id = f"test-instance-{i}"
            decision = score_and_entropy_sample(
                candidates,
                agent_role,
                instance_id,
                request_hash,
                bias_config,
            )
            decisions.append(decision)

        unique_suppliers = set(d.supplier_id for d in decisions)
        assert len(unique_suppliers) > 1

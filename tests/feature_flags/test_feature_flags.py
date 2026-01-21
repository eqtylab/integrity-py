"""Test feature flags cross-language functionality."""

import os
import unittest

from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags, feature_gate, if_feature_enabled


class TestFeatureFlags(unittest.TestCase):
    def setup_method(self):
        """Clear runtime flags before each test."""
        FeatureFlags.clear_runtime()

    def test_runtime_feature_flag(self):
        """Test setting and checking feature flags at runtime."""
        assert not FeatureFlags.is_enabled("test_feature")

        FeatureFlags.enable("test_feature")
        assert FeatureFlags.is_enabled("test_feature")

        FeatureFlags.disable("test_feature")
        assert not FeatureFlags.is_enabled("test_feature")

    def test_environment_variable_fallback(self):
        """Test that environment variables work as fallback."""
        # Clear any runtime settings
        FeatureFlags.clear_runtime()

        # Test without env var
        assert not FeatureFlags.is_enabled("env_test_feature")

        # Set environment variable
        os.environ["EQTY_FEATURE_ENV_TEST_FEATURE"] = "true"
        assert FeatureFlags.is_enabled("env_test_feature")

        # Runtime setting should override env var
        FeatureFlags.disable("env_test_feature")
        assert not FeatureFlags.is_enabled("env_test_feature")

        # Clean up
        del os.environ["EQTY_FEATURE_ENV_TEST_FEATURE"]

    def test_get_all_features(self):
        """Test getting all runtime feature flags."""
        FeatureFlags.enable("feature1")
        FeatureFlags.enable("feature2")
        FeatureFlags.disable("feature3")

        all_flags = FeatureFlags.get_all()
        assert all_flags["feature1"] is True
        assert all_flags["feature2"] is True
        assert all_flags["feature3"] is False

    def test_configure_from_dict(self):
        """Test configuring multiple features from a dictionary."""
        config = {
            "feature_a": True,
            "feature_b": False,
            "feature_c": True,
        }

        FeatureFlags.configure_from_dict(config)

        assert FeatureFlags.is_enabled("feature_a")
        assert not FeatureFlags.is_enabled("feature_b")
        assert FeatureFlags.is_enabled("feature_c")

    def test_feature_flag_context_manager(self):
        """Test temporary feature flag changes with context manager."""
        # Initially disabled
        assert not FeatureFlags.is_enabled("temp_feature")

        with FeatureFlags.with_feature("temp_feature", True):
            assert FeatureFlags.is_enabled("temp_feature")

        # Should be disabled again
        assert not FeatureFlags.is_enabled("temp_feature")

        # Test with previously set feature
        FeatureFlags.enable("existing_feature")
        assert FeatureFlags.is_enabled("existing_feature")

        with FeatureFlags.with_feature("existing_feature", False):
            assert not FeatureFlags.is_enabled("existing_feature")

        # Should be restored
        assert FeatureFlags.is_enabled("existing_feature")

    def test_feature_gate_decorator(self):
        """Test the feature gate decorator."""

        @feature_gate("test_decorator_feature")
        def gated_function():
            return "feature enabled"

        # Should raise when feature is disabled
        with self.assertRaisesRegex(
            RuntimeError, "Feature 'test_decorator_feature' is not enabled"
        ):
            gated_function()

        # Should work when feature is enabled
        FeatureFlags.enable("test_decorator_feature")
        self.assertEqual(gated_function(), "feature enabled")

    def test_feature_gate_invert(self):
        """Test the feature gate decorator with invert=True."""

        @feature_gate("invert_test_feature", invert=True)
        def inverted_function():
            return "runs when disabled"

        # Should work when feature is disabled (default state)
        self.assertEqual(inverted_function(), "runs when disabled")

        # Should raise when feature is enabled
        FeatureFlags.enable("invert_test_feature")
        with self.assertRaisesRegex(RuntimeError, "Feature 'invert_test_feature' is not disabled"):
            inverted_function()

        # Should work again when feature is disabled
        FeatureFlags.disable("invert_test_feature")
        self.assertEqual(inverted_function(), "runs when disabled")

    def test_feature_gate_when_disabled(self):
        """Test the feature_gate_when_disabled decorator."""
        from eqty_sdk.feature_flags import feature_gate_when_disabled

        @feature_gate_when_disabled("disabled_test_feature")
        def legacy_function():
            return "legacy behavior"

        # Should work when feature is disabled (default state)
        self.assertEqual(legacy_function(), "legacy behavior")

        # Should raise when feature is enabled
        FeatureFlags.enable("disabled_test_feature")
        with self.assertRaisesRegex(
            RuntimeError, "Feature 'disabled_test_feature' is not disabled"
        ):
            legacy_function()

        # Should work again when feature is disabled
        FeatureFlags.disable("disabled_test_feature")
        self.assertEqual(legacy_function(), "legacy behavior")

    def test_feature_gate_with_fallback(self):
        """Test feature gate decorator with fallback function."""

        def fallback_fn():
            return "fallback result"

        @feature_gate("fallback_test", fallback_fn=fallback_fn)
        def main_function():
            return "main result"

        # Should use fallback when disabled
        assert main_function() == "fallback result"

        # Should use main function when enabled
        FeatureFlags.enable("fallback_test")
        assert main_function() == "main result"

    def test_if_feature_enabled_helper(self):
        """Test the if_feature_enabled helper function."""
        getter = if_feature_enabled("helper_test", "default")

        # Should return default when disabled
        assert getter("enabled_value") == "default"

        # Should return value when enabled
        FeatureFlags.enable("helper_test")
        assert getter("enabled_value") == "enabled_value"

    def test_from_env_direct(self):
        """Test checking environment variables directly."""
        # Test default
        assert not FeatureFlags.from_env("direct_env_test")
        assert FeatureFlags.from_env("direct_env_test", default=True)

        # Test with env var
        os.environ["EQTY_FEATURE_DIRECT_ENV_TEST"] = "true"
        assert FeatureFlags.from_env("direct_env_test")

        # Clean up
        del os.environ["EQTY_FEATURE_DIRECT_ENV_TEST"]

    def test_case_insensitive_feature_flags(self):
        """Test that feature flags are case insensitive."""
        # Test different cases for the same feature
        FeatureFlags.enable("Test_Feature")

        # All these should be True regardless of case
        assert FeatureFlags.is_enabled("test_feature")
        assert FeatureFlags.is_enabled("TEST_FEATURE")
        assert FeatureFlags.is_enabled("Test_Feature")
        assert FeatureFlags.is_enabled("tEsT_fEaTuRe")

        # Disable with different case
        FeatureFlags.disable("TEST_FEATURE")

        # Should all be False now
        assert not FeatureFlags.is_enabled("test_feature")
        assert not FeatureFlags.is_enabled("TEST_FEATURE")
        assert not FeatureFlags.is_enabled("Test_Feature")

        # Test enum is also case insensitive when used as string
        FeatureFlags.enable("GRAPH_IDS")
        assert FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS)
        assert FeatureFlags.is_enabled("graph_ids")


def test_example_usage():
    """Test example usage patterns for documentation."""
    # Basic usage
    FeatureFlags.enable("new_computation_engine")
    assert FeatureFlags.is_enabled("new_computation_engine")

    # Environment variable
    os.environ["EQTY_FEATURE_EXPERIMENTAL_ASSETS"] = "true"
    assert FeatureFlags.is_enabled("experimental_assets")

    # Context manager
    with FeatureFlags.with_feature("temporary_mode"):
        assert FeatureFlags.is_enabled("temporary_mode")

    # Decorator
    @feature_gate("api_v2")
    def new_api_endpoint():
        return {"version": 2, "data": "new format"}

    FeatureFlags.enable("api_v2")
    result = new_api_endpoint()
    assert result["version"] == 2

    # Clean up
    FeatureFlags.clear_runtime()
    del os.environ["EQTY_FEATURE_EXPERIMENTAL_ASSETS"]

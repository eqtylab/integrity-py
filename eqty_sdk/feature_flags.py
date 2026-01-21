"""Feature flags module for eqty_sdk.

Provides cross-language feature flag management between Python and Rust components.
Supports both runtime configuration and environment variable fallback.
"""

import os
from enum import Enum
from typing import Dict, cast

from eqty_sdk._rust import feature_flags as rust_feature_flags


class FEATURE_FLAGS(Enum):
    GRAPH_IDS = "graph_ids"


class FeatureFlags:
    """Cross-language feature flag management.

    Features can be controlled at runtime via Python or read from environment variables.
    Runtime settings take precedence over environment variables.

    Environment variable format: EQTY_FEATURE_{FEATURE_NAME_UPPER}=true|false

    Example:
        # Runtime control
        FeatureFlags.enable("new_computation_engine")
        assert FeatureFlags.is_enabled("new_computation_engine") == True

        # Environment variable fallback
        os.environ["EQTY_FEATURE_EXPERIMENTAL_ASSETS"] = "true"
        assert FeatureFlags.is_enabled("experimental_assets") == True

    """

    @staticmethod
    def _get_feature_name(feature: str | FEATURE_FLAGS) -> str:
        """Convert feature flag enum or string to lowercase string."""
        name = feature.value if isinstance(feature, FEATURE_FLAGS) else feature
        return name.lower()

    @staticmethod
    def enable(feature_name: str | FEATURE_FLAGS) -> None:
        """Enable a feature flag at runtime."""
        name = FeatureFlags._get_feature_name(feature_name)
        rust_feature_flags.set_feature_flag(name, True)

    @staticmethod
    def disable(feature_name: str | FEATURE_FLAGS) -> None:
        """Disable a feature flag at runtime."""
        name = FeatureFlags._get_feature_name(feature_name)
        rust_feature_flags.set_feature_flag(name, False)

    @staticmethod
    def is_enabled(feature_name: str | FEATURE_FLAGS) -> bool:
        """Check if a feature is enabled.

        Checks runtime configuration first, then falls back to environment variable.
        Returns False if neither is set.
        """
        name = FeatureFlags._get_feature_name(feature_name)
        return cast(bool, rust_feature_flags.is_feature_enabled(name))

    @staticmethod
    def is_disabled(feature_name: str | FEATURE_FLAGS) -> bool:
        """Check if a feature is disabled.
        Calls is_enabled(feature_name) and inverts the result.
        """
        return not FeatureFlags.is_enabled(feature_name)

    @staticmethod
    def get_all() -> Dict[str, bool]:
        """Get all currently set runtime feature flags."""
        return cast(Dict[str, bool], rust_feature_flags.get_all_feature_flags())

    @staticmethod
    def clear_runtime() -> None:
        """Clear all runtime feature flags (keeps environment variables)."""
        rust_feature_flags.clear_runtime_feature_flags()

    @staticmethod
    def from_env(feature_name: str | FEATURE_FLAGS, default: bool = False) -> bool:
        """Check environment variable directly, bypassing runtime config."""
        name = FeatureFlags._get_feature_name(feature_name)
        return os.getenv(f"EQTY_FEATURE_{name.upper()}", str(default)).lower() == "true"

    @staticmethod
    def configure_from_dict(config: Dict[str, bool]) -> None:
        """Configure multiple features from a dictionary."""
        for feature, enabled in config.items():
            rust_feature_flags.set_feature_flag(feature, enabled)

    @staticmethod
    def with_feature(feature_name: str | FEATURE_FLAGS, enabled: bool = True):
        """Context manager for temporarily enabling/disabling a feature.

        Example:
            with FeatureFlags.with_feature(FEATURE_FLAGS.GRAPH_IDS):
                # Feature is enabled in this block
                do_experimental_work()
            # Feature is restored to previous state

        """
        name = FeatureFlags._get_feature_name(feature_name)
        return _FeatureFlagContext(name, enabled)


class _FeatureFlagContext:
    """Context manager for temporary feature flag changes."""

    def __init__(self, feature_name: str, enabled: bool):
        self.feature_name = feature_name
        self.enabled = enabled
        self.previous_state: bool = False  # Default to False, will be overwritten if actually set
        self.was_previously_set = False

    def __enter__(self):
        # Store current state
        all_flags = cast(Dict[str, bool], rust_feature_flags.get_all_feature_flags())
        if self.feature_name in all_flags:
            self.previous_state = all_flags[self.feature_name]
            self.was_previously_set = True
        else:
            self.previous_state = False  # Keep as False since it wasn't set
            self.was_previously_set = False

        # Set new state
        rust_feature_flags.set_feature_flag(self.feature_name, self.enabled)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        # Restore previous state
        if self.was_previously_set:
            # Restore the previous boolean value (could be True or False)
            rust_feature_flags.set_feature_flag(self.feature_name, self.previous_state)
        else:
            # Feature wasn't set before, so clear it from runtime config
            all_flags = cast(Dict[str, bool], rust_feature_flags.get_all_feature_flags())
            if self.feature_name in all_flags:
                # We can't directly remove from Rust HashMap, but we can clear all and restore others
                other_flags = {k: v for k, v in all_flags.items() if k != self.feature_name}
                rust_feature_flags.clear_runtime_feature_flags()
                for name, enabled in other_flags.items():
                    rust_feature_flags.set_feature_flag(name, enabled)


# Convenience functions for common patterns
def feature_gate(feature_name: str | FEATURE_FLAGS, fallback_fn=None, invert: bool = False):
    """Decorator to gate a function behind a feature flag.

    Args:
        feature_name: Name of the feature flag to check
        fallback_fn: Optional function to call if feature condition is not met
        invert: If True, function runs when feature is DISABLED (default: False)

    Example:
        @feature_gate("new_computation_engine")
        def enhanced_compute():
            return "new computation result"

        @feature_gate("experimental_api", fallback_fn=lambda: "not implemented")
        def experimental_endpoint():
            return "experimental result"

        @feature_gate("legacy_mode", invert=True)
        def legacy_function():
            return "runs when legacy_mode is DISABLED"

    """

    def decorator(func):
        def wrapper(*args, **kwargs):
            name = FeatureFlags._get_feature_name(feature_name)
            feature_enabled = FeatureFlags.is_enabled(feature_name)
            should_run = not feature_enabled if invert else feature_enabled

            if should_run:
                return func(*args, **kwargs)
            elif fallback_fn:
                return fallback_fn(*args, **kwargs)
            else:
                status = "disabled" if invert else "enabled"
                raise RuntimeError(f"Feature '{name}' is not {status}")

        return wrapper

    return decorator


def feature_gate_when_disabled(feature_name: str | FEATURE_FLAGS, fallback_fn=None):
    """Decorator that runs a function only when a feature flag is DISABLED.

    Args:
        feature_name: Name of the feature flag to check
        fallback_fn: Optional function to call if feature is enabled

    Example:
        @feature_gate_when_disabled("graph_id")
        def legacy_add_attribute(self, **kwargs):
            # This runs when graph_id is DISABLED
            return self

    """
    return feature_gate(feature_name, fallback_fn=fallback_fn, invert=True)


def if_feature_enabled(feature_name: str, default_value=None):
    """Return a value only if a feature is enabled.

    Example:
        result = if_feature_enabled("new_api", "enhanced_result") or "legacy_result"

    """

    def inner(value):
        return value if FeatureFlags.is_enabled(feature_name) else default_value

    return inner

use std::{collections::HashMap, env, sync::RwLock};

use once_cell::sync::Lazy;
use pyo3::prelude::*;

/// Global feature flag storage
static FEATURE_CONFIG: Lazy<RwLock<HashMap<String, bool>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Feature flags module for cross-language feature management
pub struct FeatureFlags;

impl FeatureFlags {
    /// Check if a feature is enabled, with environment variable fallback
    pub fn is_enabled(feature: &str) -> bool {
        let feature_lower = feature.to_lowercase();

        // First check runtime config
        if let Ok(config) = FEATURE_CONFIG.read() {
            if let Some(&enabled) = config.get(&feature_lower) {
                return enabled;
            }
        }

        // Fallback to environment variable
        env::var(format!("EQTY_FEATURE_{}", feature.to_uppercase()))
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true"
    }

    /// Set a feature flag at runtime
    pub fn set_feature(feature: &str, enabled: bool) {
        let feature_lower = feature.to_lowercase();
        if let Ok(mut config) = FEATURE_CONFIG.write() {
            config.insert(feature_lower, enabled);
        }
    }

    /// Get all currently set feature flags
    pub fn get_all_features() -> HashMap<String, bool> {
        FEATURE_CONFIG.read().unwrap().clone()
    }

    /// Clear all runtime feature flags (keeps environment variables)
    pub fn clear_runtime_flags() {
        if let Ok(mut config) = FEATURE_CONFIG.write() {
            config.clear();
        }
    }
}

/// Python-exposed function to set feature flag
#[pyfunction]
pub fn set_feature_flag(feature: String, enabled: bool) {
    FeatureFlags::set_feature(&feature, enabled);
}

/// Python-exposed function to check if feature is enabled
#[pyfunction]
pub fn is_feature_enabled(feature: String) -> bool {
    FeatureFlags::is_enabled(&feature)
}

/// Python-exposed function to get all feature flags
#[pyfunction]
pub fn get_all_feature_flags() -> HashMap<String, bool> {
    FeatureFlags::get_all_features()
}

/// Python-exposed function to clear runtime flags
#[pyfunction]
pub fn clear_runtime_feature_flags() {
    FeatureFlags::clear_runtime_flags();
}

/// Python module for feature flags
#[pymodule]
pub fn feature_flags(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(set_feature_flag, m)?)?;
    m.add_function(wrap_pyfunction!(is_feature_enabled, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_feature_flags, m)?)?;
    m.add_function(wrap_pyfunction!(clear_runtime_feature_flags, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag_defaults_false() {
        assert!(!FeatureFlags::is_enabled("nonexistent_feature"));
    }

    #[test]
    fn test_feature_flag_runtime_setting() {
        FeatureFlags::set_feature("test_feature", true);
        assert!(FeatureFlags::is_enabled("test_feature"));

        FeatureFlags::set_feature("test_feature", false);
        assert!(!FeatureFlags::is_enabled("test_feature"));
    }

    #[test]
    fn test_clear_runtime_flags() {
        FeatureFlags::set_feature("test_clear", true);
        assert!(FeatureFlags::is_enabled("test_clear"));

        FeatureFlags::clear_runtime_flags();
        assert!(!FeatureFlags::is_enabled("test_clear"));
    }
}

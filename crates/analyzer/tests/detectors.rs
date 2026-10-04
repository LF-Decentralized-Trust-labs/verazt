//! Unit tests for detectors.

use analyzer::{DetectorMeta, DetectorRegistry, register_all_detectors};

fn create_registry() -> DetectorRegistry {
    let mut registry = DetectorRegistry::new();
    register_all_detectors(&mut registry);
    registry
}

fn meta_of(id: &str) -> &'static DetectorMeta {
    create_registry()
        .get(id)
        .unwrap_or_else(|| panic!("{id} detector should exist"))
        .meta()
}

/// Test that all detectors have valid metadata.
#[test]
fn test_detectors_have_valid_metadata() {
    let registry = create_registry();

    for detector in registry.all() {
        let meta = detector.meta();

        // Check ID is non-empty
        assert!(!meta.id.as_str().is_empty(), "Detector ID should not be empty");

        // Check name is non-empty
        assert!(!meta.name.is_empty(), "Detector name should not be empty");

        // Check description is non-empty
        assert!(!meta.description.is_empty(), "Detector description should not be empty");

        // Check recommendation is non-empty
        assert!(
            !meta.recommendation.is_empty(),
            "Detector recommendation should not be empty for {}",
            meta.id
        );

        // The pass name and the metadata name are the same identity
        assert_eq!(detector.name(), meta.name);
    }
}

/// Test tx-origin detector metadata.
#[test]
fn test_tx_origin_detector() {
    let meta = meta_of("tx-origin");
    assert_eq!(meta.id.as_str(), "tx-origin");
    assert_eq!(meta.name, "Dangerous use of tx.origin");
    assert_eq!(meta.swc_ids, [115]);
    assert_eq!(meta.cwe_ids, [345]);
}

/// Test reentrancy detector metadata.
#[test]
fn test_reentrancy_detector() {
    let meta = meta_of("reentrancy");
    assert_eq!(meta.id.as_str(), "reentrancy");
    assert_eq!(meta.swc_ids, [107]);
}

/// Test unchecked-call detector metadata.
#[test]
fn test_unchecked_call_detector() {
    let meta = meta_of("unchecked-call");
    assert_eq!(meta.id.as_str(), "unchecked-call");
    assert_eq!(meta.swc_ids, [104]);
}

/// Test floating-pragma detector metadata.
#[test]
fn test_floating_pragma_detector() {
    let meta = meta_of("floating-pragma");
    assert_eq!(meta.id.as_str(), "floating-pragma");
    assert_eq!(meta.swc_ids, [103]);
}

/// Test shadowing detector metadata.
#[test]
fn test_shadowing_detector() {
    let meta = meta_of("shadowing");
    assert_eq!(meta.id.as_str(), "shadowing");
    assert_eq!(meta.swc_ids, [119]);
}

/// Test uninitialized detector metadata.
#[test]
fn test_uninitialized_detector() {
    let meta = meta_of("uninitialized-storage");
    assert_eq!(meta.id.as_str(), "uninitialized-storage");
    assert_eq!(meta.swc_ids, [109]);
}

/// Test deprecated detector metadata.
#[test]
fn test_deprecated_detector() {
    let meta = meta_of("deprecated");
    assert_eq!(meta.id.as_str(), "deprecated");
    assert_eq!(meta.swc_ids, [111]);
}

/// Test visibility detector metadata.
#[test]
fn test_visibility_detector() {
    let meta = meta_of("visibility");
    assert_eq!(meta.id.as_str(), "visibility");
    assert!(meta.swc_ids.contains(&100));
    assert!(meta.swc_ids.contains(&108));
}

/// Test dead-code detector metadata.
#[test]
fn test_dead_code_detector() {
    let meta = meta_of("dead-code");
    assert_eq!(meta.id.as_str(), "dead-code");
    assert!(meta.cwe_ids.contains(&561));
}

//! Unit tests for detectors.

use smarthunt::{DetectorRegistry, register_all_detectors};

fn create_registry() -> DetectorRegistry {
    let mut registry = DetectorRegistry::new();
    register_all_detectors(&mut registry);
    registry
}

/// Test that all detectors have valid metadata.
#[test]
fn test_detectors_have_valid_metadata() {
    let registry = create_registry();

    for detector in registry.all() {
        // Check ID is non-empty
        assert!(!detector.id().as_str().is_empty(), "Detector ID should not be empty");

        // Check name is non-empty
        assert!(!detector.name().is_empty(), "Detector name should not be empty");

        // Check description is non-empty
        assert!(!detector.description().is_empty(), "Detector description should not be empty");

        // Check recommendation is non-empty
        assert!(!detector.recommendation().is_empty(), "Detector recommendation should not be empty");
    }
}

/// Test tx-origin detector metadata.
#[test]
fn test_tx_origin_detector() {
    let registry = create_registry();
    let detector = registry.get("tx-origin").expect("tx-origin detector should exist");

    assert_eq!(detector.id().as_str(), "tx-origin");
    assert_eq!(detector.name(), "Dangerous use of tx.origin");
    assert_eq!(detector.swc_ids(), vec![115]);
    assert_eq!(detector.cwe_ids(), vec![345]);
}

/// Test reentrancy detector metadata.
#[test]
fn test_reentrancy_detector() {
    let registry = create_registry();
    let detector = registry.get("reentrancy").expect("reentrancy detector should exist");

    assert_eq!(detector.id().as_str(), "reentrancy");
    assert_eq!(detector.swc_ids(), vec![107]);
}

/// Test unchecked-call detector metadata.
#[test]
fn test_unchecked_call_detector() {
    let registry = create_registry();
    let detector = registry.get("unchecked-call").expect("unchecked-call detector should exist");

    assert_eq!(detector.id().as_str(), "unchecked-call");
    assert_eq!(detector.swc_ids(), vec![104]);
}

/// Test floating-pragma detector metadata.
#[test]
fn test_floating_pragma_detector() {
    let registry = create_registry();
    let detector = registry.get("floating-pragma").expect("floating-pragma detector should exist");

    assert_eq!(detector.id().as_str(), "floating-pragma");
    assert_eq!(detector.swc_ids(), vec![103]);
}

/// Test shadowing detector metadata.
#[test]
fn test_shadowing_detector() {
    let registry = create_registry();
    let detector = registry.get("shadowing").expect("shadowing detector should exist");

    assert_eq!(detector.id().as_str(), "shadowing");
    assert_eq!(detector.swc_ids(), vec![119]);
}

/// Test uninitialized detector metadata.
#[test]
fn test_uninitialized_detector() {
    let registry = create_registry();
    let detector = registry.get("uninitialized-storage").expect("uninitialized detector should exist");

    assert_eq!(detector.id().as_str(), "uninitialized-storage");
    assert_eq!(detector.swc_ids(), vec![109]);
}

/// Test deprecated detector metadata.
#[test]
fn test_deprecated_detector() {
    let registry = create_registry();
    let detector = registry.get("deprecated").expect("deprecated detector should exist");

    assert_eq!(detector.id().as_str(), "deprecated");
    assert_eq!(detector.swc_ids(), vec![111]);
}

/// Test visibility detector metadata.
#[test]
fn test_visibility_detector() {
    let registry = create_registry();
    let detector = registry.get("visibility").expect("visibility detector should exist");

    assert_eq!(detector.id().as_str(), "visibility");
    assert!(detector.swc_ids().contains(&100));
    assert!(detector.swc_ids().contains(&108));
}

/// Test dead-code detector metadata.
#[test]
fn test_dead_code_detector() {
    let registry = create_registry();
    let detector = registry.get("dead-code").expect("dead-code detector should exist");

    assert_eq!(detector.id().as_str(), "dead-code");
    assert!(detector.cwe_ids().contains(&561));
}

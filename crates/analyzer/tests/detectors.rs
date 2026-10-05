//! Unit tests for detectors.

use analyzer::{
    AnalysisConfig, AnalysisContext, DetectorMeta, DetectorRegistry, PipelineConfig,
    PipelineEngine, register_all_detectors,
};

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

/// Test that findings on EVM globals (`block.timestamp`, `tx.origin`) point
/// at the line of the global, not at an unknown location.
#[test]
fn test_evm_global_findings_have_source_location() {
    let source = "pragma solidity ^0.8.0;
contract Lottery {
    address owner;
    function draw() public view returns (bool) {
        require(tx.origin == owner);
        return block.timestamp % 7 == 0;
    }
}";
    let source_units =
        frontend::solidity::parsing::parse_solidity_source_code(source, "0.8.1").unwrap();
    let modules = frontend::solidity::lowering::lower_source_units(&source_units).unwrap();
    let mut context = AnalysisContext::new(modules, AnalysisConfig::default());
    let engine = PipelineEngine::new(PipelineConfig {
        parallel: false,
        enabled: vec!["timestamp-dependence".to_string(), "tx-origin".to_string()],
        ..PipelineConfig::default()
    });
    let result = engine.run(&mut context);

    let line_of = |id: &str| {
        let bug = result
            .bugs
            .iter()
            .find(|bug| bug.detector_id == id)
            .unwrap_or_else(|| panic!("{id} should report a bug"));
        bug.loc.start_line
    };
    assert_eq!(line_of("tx-origin"), 5);
    assert_eq!(line_of("timestamp-dependence"), 6);
}

/// Test that missing-access-control reports only the public function that
/// lets any caller change an authorization variable.
#[test]
fn test_missing_access_control_reports_unguarded_owner_change() {
    let source = "pragma solidity ^0.4.24;
contract Wallet {
    address owner;
    mapping(address => uint256) balances;
    mapping(uint256 => address) channelOwners;
    modifier onlyOwner { if (msg.sender == owner) _; }
    function Wallet() public { owner = msg.sender; }
    function deposit() public payable { balances[msg.sender] += msg.value; }
    function setOwner(address o) public onlyOwner { owner = o; }
    function resetOwner(address o) public { require(owner == msg.sender); owner = o; }
    function claim(uint256 id) public { require(channelOwners[id] == msg.sender); channelOwners[id] = 0; }
    function changeOwner(address o) public { owner = o; }
}";
    let source_units =
        frontend::solidity::parsing::parse_solidity_source_code(source, "0.4.26").unwrap();
    let modules = frontend::solidity::lowering::lower_source_units(&source_units).unwrap();
    let mut context = AnalysisContext::new(modules, AnalysisConfig::default());
    let engine = PipelineEngine::new(PipelineConfig {
        parallel: false,
        enabled: vec!["missing-access-control".to_string()],
        ..PipelineConfig::default()
    });
    let result = engine.run(&mut context);

    let lines: Vec<usize> = result.bugs.iter().map(|bug| bug.loc.start_line).collect();
    assert_eq!(lines, vec![12], "only changeOwner should be reported");
}

/// Test that bad-randomness sees Solidity 0.4 randomness sources: a
/// multi-argument `keccak256` hashing `block.timestamp`, and
/// `block.blockhash`.
#[test]
fn test_bad_randomness_detects_solidity_0_4_sources() {
    let source = "pragma solidity ^0.4.24;
contract Lottery {
    function draw() public view returns (bytes32) { return keccak256(msg.sender, block.timestamp); }
    function pick() public view returns (bytes32) { return block.blockhash(1); }
}";
    let source_units =
        frontend::solidity::parsing::parse_solidity_source_code(source, "0.4.26").unwrap();
    let modules = frontend::solidity::lowering::lower_source_units(&source_units).unwrap();
    let mut context = AnalysisContext::new(modules, AnalysisConfig::default());
    let engine = PipelineEngine::new(PipelineConfig {
        parallel: false,
        enabled: vec!["bad-randomness".to_string()],
        ..PipelineConfig::default()
    });
    let result = engine.run(&mut context);

    let mut lines: Vec<usize> = result.bugs.iter().map(|bug| bug.loc.start_line).collect();
    lines.dedup();
    assert_eq!(lines, vec![3, 4]);
}

/// Test that unchecked-call reports discarded results of `call.value(...)`
/// and `send`, and not checked calls or `transfer`, which reverts.
#[test]
fn test_unchecked_call_reports_discarded_call_results() {
    let source = "pragma solidity ^0.4.24;
contract Payer {
    function a(address t) public { t.call.value(1)(\"\"); }
    function b(address t) public { t.send(1); }
    function c(address t) public { require(t.call.value(1)(\"\")); }
    function d(address t) public { t.transfer(1); }
}";
    let source_units =
        frontend::solidity::parsing::parse_solidity_source_code(source, "0.4.26").unwrap();
    let modules = frontend::solidity::lowering::lower_source_units(&source_units).unwrap();
    let mut context = AnalysisContext::new(modules, AnalysisConfig::default());
    let engine = PipelineEngine::new(PipelineConfig {
        parallel: false,
        enabled: vec!["unchecked-call".to_string()],
        ..PipelineConfig::default()
    });
    let result = engine.run(&mut context);

    let mut lines: Vec<usize> = result.bugs.iter().map(|bug| bug.loc.start_line).collect();
    lines.sort();
    assert_eq!(lines, vec![3, 4]);
}

//! Matcher module: compare ground truth annotations vs tool detections.
//!
//! Matching is done by **(file, BugCategory)**.

use crate::annotation::AnnotatedBug;
use crate::runner::DetectedBug;
use bugs::bug::BugCategory;
use std::path::{Path, PathBuf};

/// A matched bug (true positive).
#[derive(Debug, Clone)]
pub struct MatchedBug {
    /// The annotated ground truth bug.
    pub annotation: AnnotatedBug,
    /// The detected bug that matched.
    pub detection: DetectedBug,
}

/// Result of matching annotations vs detections for a single file.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Path to the file.
    pub file_path: PathBuf,
    /// True positives: annotated AND detected.
    pub true_positives: Vec<MatchedBug>,
    /// False negatives: annotated BUT NOT detected.
    pub false_negatives: Vec<AnnotatedBug>,
    /// False positives: detected BUT NOT annotated.
    pub false_positives: Vec<DetectedBug>,
}

/// Compare annotations vs detections for a single file.
///
/// Matching logic:
/// - For each annotated category in the file, check if there's a detection with
///   the same category.
/// - Each detection can only match one annotation (greedy matching).
pub fn match_file(
    file_path: &Path,
    annotations: &[AnnotatedBug],
    detections: &[DetectedBug],
) -> MatchResult {
    let mut true_positives = Vec::new();
    let mut false_negatives = Vec::new();

    // Track which detections have been matched
    let mut matched_detection_indices: Vec<bool> = vec![false; detections.len()];

    // For each annotation, try to find a matching detection
    for annotation in annotations {
        let mut found = false;
        for (idx, detection) in detections.iter().enumerate() {
            if !matched_detection_indices[idx]
                && categories_match(annotation.category, detection.category)
            {
                true_positives.push(MatchedBug {
                    annotation: annotation.clone(),
                    detection: detection.clone(),
                });
                matched_detection_indices[idx] = true;
                found = true;
                break;
            }
        }
        if !found {
            false_negatives.push(annotation.clone());
        }
    }

    // All unmatched detections are false positives
    let false_positives: Vec<DetectedBug> = detections
        .iter()
        .enumerate()
        .filter(|(idx, _)| !matched_detection_indices[*idx])
        .map(|(_, d)| d.clone())
        .collect();

    MatchResult {
        file_path: file_path.to_path_buf(),
        true_positives,
        false_negatives,
        false_positives,
    }
}

/// Check if two categories match.
///
/// This allows some flexibility in matching - e.g., CodeQuality detections
/// are not compared against vulnerability annotations.
fn categories_match(annotated: BugCategory, detected: BugCategory) -> bool {
    annotated == detected
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_annotation(category: BugCategory) -> AnnotatedBug {
        AnnotatedBug {
            file_path: PathBuf::from("test.sol"),
            line_number: 10,
            bug_line: 11,
            bug_name: category.to_annotation().to_string(),
            category,
        }
    }

    fn make_detection(category: BugCategory) -> DetectedBug {
        DetectedBug {
            name: "Test Detector".to_string(),
            category,
            start_line: 11,
            severity: "High".to_string(),
        }
    }

    #[test]
    fn test_match_file_all_tp() {
        let annotations = vec![make_annotation(BugCategory::Reentrancy)];
        let detections = vec![make_detection(BugCategory::Reentrancy)];

        let result = match_file(Path::new("test.sol"), &annotations, &detections);

        assert_eq!(result.true_positives.len(), 1);
        assert!(result.false_negatives.is_empty());
        assert!(result.false_positives.is_empty());
    }

    #[test]
    fn test_match_file_false_negative() {
        let annotations = vec![make_annotation(BugCategory::Reentrancy)];
        let detections: Vec<DetectedBug> = vec![];

        let result = match_file(Path::new("test.sol"), &annotations, &detections);

        assert!(result.true_positives.is_empty());
        assert_eq!(result.false_negatives.len(), 1);
        assert!(result.false_positives.is_empty());
    }

    #[test]
    fn test_match_file_false_positive() {
        let annotations: Vec<AnnotatedBug> = vec![];
        let detections = vec![make_detection(BugCategory::Reentrancy)];

        let result = match_file(Path::new("test.sol"), &annotations, &detections);

        assert!(result.true_positives.is_empty());
        assert!(result.false_negatives.is_empty());
        assert_eq!(result.false_positives.len(), 1);
    }

    #[test]
    fn test_match_file_mixed() {
        let annotations = vec![
            make_annotation(BugCategory::Reentrancy),
            make_annotation(BugCategory::AccessControl),
        ];
        let detections = vec![
            make_detection(BugCategory::Reentrancy),
            make_detection(BugCategory::TimeManipulation),
        ];

        let result = match_file(Path::new("test.sol"), &annotations, &detections);

        assert_eq!(result.true_positives.len(), 1);
        assert_eq!(result.false_negatives.len(), 1);
        assert_eq!(result.false_positives.len(), 1);
        assert_eq!(result.false_negatives[0].category, BugCategory::AccessControl);
        assert_eq!(result.false_positives[0].category, BugCategory::TimeManipulation);
    }

    #[test]
    fn test_match_file_multiple_same_category() {
        let annotations = vec![
            make_annotation(BugCategory::AccessControl),
            make_annotation(BugCategory::AccessControl),
        ];
        let detections = vec![make_detection(BugCategory::AccessControl)];

        let result = match_file(Path::new("test.sol"), &annotations, &detections);

        assert_eq!(result.true_positives.len(), 1);
        assert_eq!(result.false_negatives.len(), 1);
        assert!(result.false_positives.is_empty());
    }

    #[test]
    fn test_match_file_category_mismatch() {
        let annotations = vec![make_annotation(BugCategory::Reentrancy)];
        let detections = vec![make_detection(BugCategory::AccessControl)];

        let result = match_file(Path::new("test.sol"), &annotations, &detections);

        assert!(result.true_positives.is_empty());
        assert_eq!(result.false_negatives.len(), 1);
        assert_eq!(result.false_positives.len(), 1);
    }

    #[test]
    fn test_match_file_empty() {
        let result = match_file(Path::new("test.sol"), &[], &[]);

        assert!(result.true_positives.is_empty());
        assert!(result.false_negatives.is_empty());
        assert!(result.false_positives.is_empty());
    }
}

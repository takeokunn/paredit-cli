use super::super::types::check::RefactorCheckResult;
use super::super::types::status::{RefactorStatusBlockedReason, RefactorStatusNextAction};

pub(in crate::presentation::cli) fn refactor_status_blocked_reasons(
    check: &RefactorCheckResult,
) -> Vec<RefactorStatusBlockedReason> {
    let mut reasons = Vec::new();

    if !check.manifest_policy_passed {
        reasons.push(RefactorStatusBlockedReason::ManifestPolicyFailed);
    }
    if !check.manifest_outputs_parse {
        reasons.push(RefactorStatusBlockedReason::ManifestOutputsDoNotParse);
    }
    if check.summary.stale_file_count > 0 {
        reasons.push(RefactorStatusBlockedReason::StaleFiles);
    }
    if check.summary.output_hash_mismatch_count > 0 {
        reasons.push(RefactorStatusBlockedReason::OutputHashMismatches);
    }
    if check.summary.parse_error_count > 0 {
        reasons.push(RefactorStatusBlockedReason::ParseErrors);
    }
    if check.summary.manifest_flag_mismatch_count > 0 {
        reasons.push(RefactorStatusBlockedReason::ManifestFlagMismatches);
    }

    reasons
}

pub(in crate::presentation::cli) fn refactor_status_next_action(
    reasons: &[RefactorStatusBlockedReason],
) -> RefactorStatusNextAction {
    if reasons.is_empty() {
        return RefactorStatusNextAction::RunDiffThenApplyWrite;
    }

    if reasons.iter().any(|reason| {
        matches!(
            reason,
            RefactorStatusBlockedReason::ManifestPolicyFailed
                | RefactorStatusBlockedReason::ManifestOutputsDoNotParse
                | RefactorStatusBlockedReason::StaleFiles
                | RefactorStatusBlockedReason::ManifestFlagMismatches
        )
    }) {
        return RefactorStatusNextAction::RegeneratePreview;
    }

    RefactorStatusNextAction::FixManifestOrParser
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use proptest::prelude::*;

    use super::*;
    use crate::presentation::cli::refactor::types::check::RefactorCheckSummary;
    use crate::presentation::cli::refactor::types::manifest::RefactorApplyManifestHeader;
    use crate::presentation::cli::refactor::types::root::RefactorRootReport;

    fn check_result(
        manifest_policy_passed: bool,
        manifest_outputs_parse: bool,
        stale_file_count: usize,
        output_hash_mismatch_count: usize,
        parse_error_count: usize,
        manifest_flag_mismatch_count: usize,
    ) -> RefactorCheckResult {
        RefactorCheckResult {
            manifest: RefactorApplyManifestHeader {
                path: PathBuf::from("refactor-preview.json"),
                hash: "manifest-hash".to_string(),
                mode: "rename-symbol".to_string(),
                from: "old-name".to_string(),
                to: "new-name".to_string(),
            },
            root: RefactorRootReport {
                enforced: false,
                path: None,
            },
            manifest_policy_passed,
            manifest_outputs_parse,
            files: Vec::new(),
            summary: RefactorCheckSummary {
                file_count: 0,
                changed_file_count: 0,
                edit_count: 0,
                stale_file_count,
                output_hash_mismatch_count,
                parse_error_count,
                manifest_flag_mismatch_count,
                can_apply: manifest_policy_passed
                    && manifest_outputs_parse
                    && stale_file_count == 0
                    && output_hash_mismatch_count == 0
                    && parse_error_count == 0
                    && manifest_flag_mismatch_count == 0,
            },
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(96))]

        #[test]
        fn pbt_blocked_reasons_are_exactly_the_failed_check_inputs(
            manifest_policy_passed in any::<bool>(),
            manifest_outputs_parse in any::<bool>(),
            stale_file_count in 0usize..4,
            output_hash_mismatch_count in 0usize..4,
            parse_error_count in 0usize..4,
            manifest_flag_mismatch_count in 0usize..4,
        ) {
            let check = check_result(
                manifest_policy_passed,
                manifest_outputs_parse,
                stale_file_count,
                output_hash_mismatch_count,
                parse_error_count,
                manifest_flag_mismatch_count,
            );
            let reasons = refactor_status_blocked_reasons(&check);

            prop_assert_eq!(
                reasons.contains(&RefactorStatusBlockedReason::ManifestPolicyFailed),
                !manifest_policy_passed
            );
            prop_assert_eq!(
                reasons.contains(&RefactorStatusBlockedReason::ManifestOutputsDoNotParse),
                !manifest_outputs_parse
            );
            prop_assert_eq!(
                reasons.contains(&RefactorStatusBlockedReason::StaleFiles),
                stale_file_count > 0
            );
            prop_assert_eq!(
                reasons.contains(&RefactorStatusBlockedReason::OutputHashMismatches),
                output_hash_mismatch_count > 0
            );
            prop_assert_eq!(
                reasons.contains(&RefactorStatusBlockedReason::ParseErrors),
                parse_error_count > 0
            );
            prop_assert_eq!(
                reasons.contains(&RefactorStatusBlockedReason::ManifestFlagMismatches),
                manifest_flag_mismatch_count > 0
            );
            prop_assert_eq!(reasons.is_empty(), check.summary.can_apply);
        }

        #[test]
        fn pbt_next_action_priority_is_stable(
            manifest_policy_failed in any::<bool>(),
            manifest_outputs_do_not_parse in any::<bool>(),
            stale_files in any::<bool>(),
            output_hash_mismatches in any::<bool>(),
            parse_errors in any::<bool>(),
            manifest_flag_mismatches in any::<bool>(),
        ) {
            let mut reasons = Vec::new();
            if manifest_policy_failed {
                reasons.push(RefactorStatusBlockedReason::ManifestPolicyFailed);
            }
            if manifest_outputs_do_not_parse {
                reasons.push(RefactorStatusBlockedReason::ManifestOutputsDoNotParse);
            }
            if stale_files {
                reasons.push(RefactorStatusBlockedReason::StaleFiles);
            }
            if output_hash_mismatches {
                reasons.push(RefactorStatusBlockedReason::OutputHashMismatches);
            }
            if parse_errors {
                reasons.push(RefactorStatusBlockedReason::ParseErrors);
            }
            if manifest_flag_mismatches {
                reasons.push(RefactorStatusBlockedReason::ManifestFlagMismatches);
            }

            let expected = if reasons.is_empty() {
                RefactorStatusNextAction::RunDiffThenApplyWrite
            } else if manifest_policy_failed
                || manifest_outputs_do_not_parse
                || stale_files
                || manifest_flag_mismatches
            {
                RefactorStatusNextAction::RegeneratePreview
            } else {
                RefactorStatusNextAction::FixManifestOrParser
            };

            prop_assert_eq!(refactor_status_next_action(&reasons), expected);
        }
    }
}

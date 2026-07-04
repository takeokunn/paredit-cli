use super::super::super::super::*;
use super::super::super::args::RefactorStatusArgs;
use super::super::super::manifest::check::build_refactor_check_result;
use super::super::super::manifest::status::{
    refactor_status_blocked_reasons, refactor_status_next_action,
};
use super::super::super::render::print_refactor_status_result;
use super::super::super::types::status::{
    RefactorStatusKind, RefactorStatusResult, RefactorStatusWriteTarget,
};

pub(in crate::presentation::cli) fn refactor_status(args: RefactorStatusArgs) -> Result<()> {
    let check = build_refactor_check_result(
        &args.manifest,
        args.root.as_deref(),
        args.expect_manifest_hash.as_deref(),
    )?;
    let blocked_reasons = refactor_status_blocked_reasons(&check);
    let status = if blocked_reasons.is_empty() {
        RefactorStatusKind::Ready
    } else {
        RefactorStatusKind::Blocked
    };
    let next_action = refactor_status_next_action(&blocked_reasons);
    let write_plan = if check.summary.can_apply {
        check
            .files
            .iter()
            .filter(|file| file.changed)
            .map(|file| RefactorStatusWriteTarget {
                path: file.path.clone(),
                edit_count: file.edit_count,
                input_hash: file.input_hash.clone(),
                output_hash: file.output_hash.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };
    let result = RefactorStatusResult {
        manifest: check.manifest,
        root: check.root,
        manifest_policy_passed: check.manifest_policy_passed,
        manifest_outputs_parse: check.manifest_outputs_parse,
        status,
        next_action,
        blocked_reasons,
        write_plan,
        files: check.files,
        summary: check.summary,
    };

    print_refactor_status_result(&result, args.output)
}

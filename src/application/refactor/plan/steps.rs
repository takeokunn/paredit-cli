use std::path::PathBuf;

use super::types::{RefactorOperation, RefactorPlanGate, RefactorPlanStep};

pub fn refactor_plan_steps(
    operation: RefactorOperation,
    symbol: &str,
    files: &[PathBuf],
    gates: &[RefactorPlanGate],
) -> Vec<RefactorPlanStep> {
    let file_args = shell_file_args(files);
    let symbol_arg = shell_quote(symbol);
    let impact_command = gated_impact_report_command(&symbol_arg, &file_args);
    let mut steps = vec![
        RefactorPlanStep {
            order: 1,
            action: "run-impact-report",
            rationale: "Validate definitions, references, call graph edges, and signature compatibility before editing."
                .to_owned(),
            command: Some(impact_command.clone()),
        },
        RefactorPlanStep {
            order: 2,
            action: "run-dependency-report",
            rationale: "Check package, ASDF, load, and qualified-symbol dependencies that can invalidate a cross-file refactor."
                .to_owned(),
            command: Some(format!("paredit dependency-report --output json {file_args}")),
        },
    ];

    let blocked = gates.iter().any(|gate| gate.blocks_automation);
    let has_non_call_references = gates.iter().any(|gate| gate.code == "non-call-references");
    let (action, rationale, command) = match operation {
        RefactorOperation::Rename if blocked => (
            "review-rename-scope",
            "Blocking gates were found; inspect exact references before choosing function-only or atom-wide rename.",
            None,
        ),
        RefactorOperation::Rename if has_non_call_references => (
            "apply-symbol-rename",
            "Non-call references exist; use atom-wide rename after reviewing the exact impact scope.",
            Some(format!(
                "paredit rename-symbols --from {symbol_arg} --to <new-symbol> --output json {file_args}"
            )),
        ),
        RefactorOperation::Rename => (
            "apply-rename",
            "No blocking rename gates were found; use callable rename when every reference is a call/definition.",
            Some(format!(
                "paredit rename-function --from {symbol_arg} --to <new-symbol> --output json {file_args}"
            )),
        ),
        RefactorOperation::Remove if blocked => (
            "review-remove-scope",
            "Removal has live references or ambiguous definitions; update callers before deleting the definition.",
            None,
        ),
        RefactorOperation::Remove => (
            "apply-remove",
            "No blocking removal gates were found; remove the selected top-level definition with a dry-run plan first.",
            Some("paredit remove-definition --file <file> --path <definition-path> --plan --output json".to_owned()),
        ),
        RefactorOperation::Move if blocked => (
            "review-move-scope",
            "Move has callers or ambiguity; inspect exports, packages, and load order before moving forms.",
            None,
        ),
        RefactorOperation::Move => (
            "apply-move",
            "No blocking move gates were found; move the selected top-level definition with a dry-run plan first.",
            Some("paredit move-definition --from-file <file> --to-file <file> --path <definition-path> --plan --output json".to_owned()),
        ),
        RefactorOperation::Signature if blocked => (
            "review-signature-scope",
            "Signature changes require manual review when call arity, non-call references, or callers are unsafe.",
            None,
        ),
        RefactorOperation::Signature => (
            "apply-signature-change",
            "No blocking signature gates were found; use the dedicated parameter-edit commands with plans first.",
            Some("paredit add-function-parameter --file <file> --path <definition-path> --name <parameter> --plan --output json".to_owned()),
        ),
    };

    steps.push(RefactorPlanStep {
        order: 3,
        action,
        rationale: rationale.to_owned(),
        command,
    });
    steps.push(RefactorPlanStep {
        order: 4,
        action: "verify-after-edit",
        rationale: "Re-run impact, dependency, signature, and workspace checks after the edit to detect regressions."
            .to_owned(),
        command: Some(format!(
            "{impact_command} && paredit dependency-report --output json {file_args}"
        )),
    });

    steps
}

fn gated_impact_report_command(symbol_arg: &str, file_args: &str) -> String {
    format!(
        "paredit impact-report --symbol {symbol_arg} --fail-on-risk-level warning --require-definitions 1 --require-references 1 --require-calls 1 --output json {file_args}"
    )
}

fn shell_file_args(files: &[PathBuf]) -> String {
    files
        .iter()
        .map(|file| shell_quote(&file.display().to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

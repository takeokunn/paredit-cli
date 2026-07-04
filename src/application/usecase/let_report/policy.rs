use super::types::{LetFormReport, LetReportPolicy, LetReportPolicyOptions};

pub fn evaluate_let_report_policy(
    reports: &[LetFormReport],
    options: &LetReportPolicyOptions,
) -> LetReportPolicy {
    let mut binding_count = 0;
    let mut inlineable_binding_count = 0;
    let mut unused_binding_count = 0;
    let mut duplicate_evaluation_count = 0;

    for binding in reports.iter().flat_map(|report| &report.bindings) {
        binding_count += 1;
        if binding.can_inline_without_duplication {
            inlineable_binding_count += 1;
        }
        if binding.risks.contains(&"unused-binding") {
            unused_binding_count += 1;
        }
        if binding.risks.contains(&"duplicate-evaluation") {
            duplicate_evaluation_count += 1;
        }
    }

    let mut violations = Vec::new();
    if options.fail_on_duplicate_evaluation && duplicate_evaluation_count > 0 {
        violations.push(format!(
            "duplicate_evaluation_count {duplicate_evaluation_count} exceeds 0"
        ));
    }
    if options.fail_on_unused_binding && unused_binding_count > 0 {
        violations.push(format!(
            "unused_binding_count {unused_binding_count} exceeds 0"
        ));
    }
    if let Some(required) = options.require_inlineable_bindings {
        if inlineable_binding_count < required {
            violations.push(format!(
                "inlineable_binding_count {inlineable_binding_count} is below required {required}"
            ));
        }
    }

    LetReportPolicy {
        fail_on_duplicate_evaluation: options.fail_on_duplicate_evaluation,
        fail_on_unused_binding: options.fail_on_unused_binding,
        require_inlineable_bindings: options.require_inlineable_bindings,
        binding_count,
        inlineable_binding_count,
        unused_binding_count,
        duplicate_evaluation_count,
        passed: violations.is_empty(),
        violations,
    }
}

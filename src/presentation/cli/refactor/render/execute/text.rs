use super::super::super::super::*;
use super::super::super::types::execute::WorkspaceRefactorExecute;
use super::super::write_plan::print_refactor_write_plan;
use crate::application::refactor::execute::RefactorExecuteDecision;

pub(super) fn print_workspace_refactor_execute_text(
    execution: &WorkspaceRefactorExecute,
) -> Result<()> {
    let preview = &execution.preview;
    let write_plan = preview.write_plan();
    let writable_files = preview.writable_paths_for_write_plan(&write_plan);
    let refused_files = preview.refused_paths_for_write_plan(&write_plan);
    println!("command\tworkspace-refactor-execute");
    println!("mode\t{}", preview.mode.label());
    println!("from\t{}", preview.from);
    println!("to\t{}", preview.to);
    println!("write_requested\t{}", preview.write_requested);
    print_refactor_write_plan(&write_plan, &writable_files, &refused_files);
    print_decision("preflight_decision", &execution.preflight_decision);
    print_decision("execute_decision", &execution.execute_decision);
    println!("changed_file_count\t{}", preview.summary.changed_file_count);
    for changed_file in &preview.summary.changed_files {
        println!("changed_file\t{changed_file}");
    }
    println!(
        "unchanged_file_count\t{}",
        preview.summary.unchanged_file_count
    );
    println!("written_file_count\t{}", preview.summary.written_file_count);
    println!("edit_count\t{}", preview.summary.edit_count);
    println!("parse_error_count\t{}", preview.summary.parse_error_count);
    println!("policy_passed\t{}", preview.policy.passed);
    if let Some(verification) = &execution.pre_verification {
        println!("pre_verification_passed\t{}", verification.passed);
        for check in &verification.checks {
            println!(
                "pre_check\t{}\t{}\tpassed={}\tcount={}\t{}",
                check.level.label(),
                check.code,
                check.passed,
                check.count,
                check.message
            );
        }
    }
    if let Some(verification) = &execution.post_verification {
        println!("post_verification_passed\t{}", verification.passed);
        for check in &verification.checks {
            println!(
                "post_check\t{}\t{}\tpassed={}\tcount={}\t{}",
                check.level.label(),
                check.code,
                check.passed,
                check.count,
                check.message
            );
        }
    }

    Ok(())
}

fn print_decision(label: &str, decision: &RefactorExecuteDecision) {
    println!("{}\tstatus\t{}", label, decision.status.label());
    println!("{}\treason\t{}", label, decision.status.reason());
    println!("{}\tnext_action\t{}", label, decision.status.next_action());
    for step in decision.steps() {
        println!("{}\tstep\t{}\t{}", label, step.name, step.status.label());
    }
    println!(
        "{}\twrite_parse_refused\t{}",
        label, decision.write_parse_refused
    );
    println!(
        "{}\trun_pre_verification\t{}",
        label, decision.run_pre_verification
    );
    println!("{}\tapply_preview\t{}", label, decision.apply_preview);
    println!(
        "{}\trun_post_verification\t{}",
        label, decision.run_post_verification
    );
}

use super::super::super::super::*;
use super::super::super::types::execute::WorkspaceRefactorExecute;

pub(super) fn print_workspace_refactor_execute_text(
    execution: &WorkspaceRefactorExecute,
) -> Result<()> {
    let preview = &execution.preview;
    println!("command\tworkspace-refactor-execute");
    println!("mode\t{}", preview.mode.label());
    println!("from\t{}", preview.from);
    println!("to\t{}", preview.to);
    println!("write_requested\t{}", preview.write_requested);
    println!("changed_file_count\t{}", preview.summary.changed_file_count);
    println!("written_file_count\t{}", preview.summary.written_file_count);
    println!("edit_count\t{}", preview.summary.edit_count);
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

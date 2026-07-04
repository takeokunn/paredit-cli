use super::super::super::*;
use super::super::types::plan::RefactorPlan;

pub(in crate::presentation::cli) fn print_refactor_plan(
    plan: &RefactorPlan,
    output: OutputFormat,
) -> Result<()> {
    let mut summary = summarize_impact_reports(&plan.files);
    summary.safe_to_automate = !plan.gates.iter().any(|gate| gate.blocks_automation);

    match output {
        OutputFormat::Text => {
            println!("operation\t{}", plan.operation.label());
            println!("symbol\t{}", plan.symbol);
            if let Some(workspace) = &plan.workspace {
                println!(
                    "workspace_roots\t{}",
                    workspace
                        .roots
                        .iter()
                        .map(|root| root.display().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                println!(
                    "workspace_discovered_file_count\t{}",
                    workspace.discovered_file_count
                );
                println!(
                    "workspace_skipped_unknown_count\t{}",
                    workspace.skipped_unknown_count
                );
                println!(
                    "workspace_skipped_hidden_count\t{}",
                    workspace.skipped_hidden_count
                );
                println!(
                    "workspace_skipped_generated_count\t{}",
                    workspace.skipped_generated_count
                );
                println!(
                    "workspace_skipped_symlink_count\t{}",
                    workspace.skipped_symlink_count
                );
            }
            println!("safe_to_automate\t{}", summary.safe_to_automate);
            println!("policy_passed\t{}", plan.policy.passed);
            println!(
                "policy_blocking_gate_count\t{}",
                plan.policy.blocking_gate_count
            );
            println!("policy_definition_count\t{}", plan.policy.definition_count);
            println!("policy_reference_count\t{}", plan.policy.reference_count);
            for violation in &plan.policy.violations {
                println!("policy_violation\t{violation}");
            }
            println!("files\t{}", summary.file_count);
            println!("definition_count\t{}", summary.definition_count);
            println!("reference_count\t{}", summary.reference_count);
            println!("call_count\t{}", summary.call_count);
            println!("inbound_edge_count\t{}", summary.inbound_edge_count);
            println!("outbound_edge_count\t{}", summary.outbound_edge_count);
            println!(
                "non_call_reference_count\t{}",
                summary.non_call_reference_count
            );
            println!(
                "signature_mismatch_count\t{}",
                summary.signature_mismatch_count
            );
            for gate in &plan.gates {
                println!(
                    "gate\t{}\t{}\tcount={}\tblocks={}\t{}",
                    gate.level.label(),
                    gate.code,
                    gate.count,
                    gate.blocks_automation,
                    gate.message
                );
            }
            for step in &plan.steps {
                println!(
                    "step\t{}\t{}\tcommand={}\t{}",
                    step.order,
                    step.action,
                    step.command.as_deref().unwrap_or("<manual-review>"),
                    step.rationale
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "operation": plan.operation.label(),
                "symbol": plan.symbol.as_str(),
                "workspace": plan.workspace.as_ref().map(|workspace| json!({
                    "roots": workspace
                        .roots
                        .iter()
                        .map(|root| root.display().to_string())
                        .collect::<Vec<_>>(),
                    "discovered_file_count": workspace.discovered_file_count,
                    "skipped": {
                        "unknown": workspace.skipped_unknown_count,
                        "hidden": workspace.skipped_hidden_count,
                        "generated": workspace.skipped_generated_count,
                        "symlink": workspace.skipped_symlink_count,
                    },
                })),
                "safe_to_automate": summary.safe_to_automate,
                "summary": {
                    "file_count": summary.file_count,
                    "definition_count": summary.definition_count,
                    "reference_count": summary.reference_count,
                    "call_count": summary.call_count,
                    "inbound_edge_count": summary.inbound_edge_count,
                    "outbound_edge_count": summary.outbound_edge_count,
                    "non_call_reference_count": summary.non_call_reference_count,
                    "signature_mismatch_count": summary.signature_mismatch_count,
                },
                "policy": {
                    "fail_on_blocking_gate": plan.policy.fail_on_blocking_gate,
                    "require_definitions": plan.policy.require_definitions,
                    "require_references": plan.policy.require_references,
                    "blocking_gate_count": plan.policy.blocking_gate_count,
                    "definition_count": plan.policy.definition_count,
                    "reference_count": plan.policy.reference_count,
                    "passed": plan.policy.passed,
                    "violations": plan.policy.violations,
                },
                "gates": plan
                    .gates
                    .iter()
                    .map(|gate| json!({
                        "level": gate.level.label(),
                        "code": gate.code,
                        "message": gate.message.as_str(),
                        "count": gate.count,
                        "blocks_automation": gate.blocks_automation,
                    }))
                    .collect::<Vec<_>>(),
                "steps": plan
                    .steps
                    .iter()
                    .map(|step| json!({
                        "order": step.order,
                        "action": step.action,
                        "rationale": step.rationale.as_str(),
                        "command": step.command.as_deref(),
                    }))
                    .collect::<Vec<_>>(),
                "files": plan
                    .files
                    .iter()
                    .map(|report| json!({
                        "path": report.path.display().to_string(),
                        "dialect": report.dialect.label(),
                        "package": report.package.as_deref(),
                        "definition_count": report.definitions.len(),
                        "reference_count": report.references.len(),
                        "call_count": report.calls.len(),
                        "inbound_edge_count": report.inbound_edges.len(),
                        "outbound_edge_count": report.outbound_edges.len(),
                        "non_call_reference_count": report.non_call_reference_count,
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}

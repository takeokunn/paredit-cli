use assert_cmd::Command;
use predicates::prelude::*;
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;

#[path = "cli/analysis_report.rs"]
mod analysis_report;
#[path = "cli/call_graph_report.rs"]
mod call_graph_report;
#[path = "cli/call_report.rs"]
mod call_report;
#[path = "cli/definition_movement.rs"]
mod definition_movement;
#[path = "cli/definition_removal.rs"]
mod definition_removal;
#[path = "cli/definition_report.rs"]
mod definition_report;
#[path = "cli/dependency_report.rs"]
mod dependency_report;
#[path = "cli/duplicate_report.rs"]
mod duplicate_report;
#[path = "cli/extract_function.rs"]
mod extract_function;
#[path = "cli/form_report.rs"]
mod form_report;
#[path = "cli/function_parameter.rs"]
mod function_parameter;
#[path = "cli/impact_report.rs"]
mod impact_report;
#[path = "cli/inline_function.rs"]
mod inline_function;
#[path = "cli/let_refactor.rs"]
mod let_refactor;
#[path = "cli/package/mod.rs"]
mod package;
#[path = "cli/refactor_manifest/mod.rs"]
mod refactor_manifest;
#[path = "cli/refactor_preview.rs"]
mod refactor_preview;
#[path = "cli/refactor_workspace.rs"]
mod refactor_workspace;
#[path = "cli/remove_unused_binding.rs"]
mod remove_unused_binding;
#[path = "cli/rename/mod.rs"]
mod rename;
#[path = "cli/replace_forms.rs"]
mod replace_forms;
#[path = "cli/signature_report.rs"]
mod signature_report;
#[path = "cli/sort_definitions.rs"]
mod sort_definitions;
#[path = "cli/split_file.rs"]
mod split_file;
#[path = "cli/symbol_report.rs"]
mod symbol_report;
#[path = "cli/thread_expression.rs"]
mod thread_expression;
#[path = "cli/unwrap_call.rs"]
mod unwrap_call;
#[path = "cli/workspace_report.rs"]
mod workspace_report;

fn paredit() -> Command {
    Command::cargo_bin("paredit").expect("binary")
}

fn stable_manifest_hash(text: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn fresh_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "paredit-cli-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

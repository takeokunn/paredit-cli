use assert_cmd::Command;
use predicates::prelude::*;
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, FileFailurePersistence};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "cli/analysis_report.rs"]
mod analysis_report;
#[path = "cli/bug_report_contract.rs"]
mod bug_report_contract;
#[path = "cli/call_graph_report/mod.rs"]
mod call_graph_report;
#[path = "cli/call_report.rs"]
mod call_report;
#[path = "cli/changelog_contract.rs"]
mod changelog_contract;
#[path = "cli/compatibility_docs_contract.rs"]
mod compatibility_docs_contract;
#[path = "cli/conduct_docs_contract.rs"]
mod conduct_docs_contract;
#[path = "cli/contributing_contract.rs"]
mod contributing_contract;
#[path = "cli/crate_metadata_contract.rs"]
mod crate_metadata_contract;
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
#[path = "cli/extract_function/mod.rs"]
mod extract_function;
#[path = "cli/feature_request_contract.rs"]
mod feature_request_contract;
#[path = "cli/form_report.rs"]
mod form_report;
#[path = "cli/format/mod.rs"]
mod format;
#[path = "cli/function_parameter/mod.rs"]
mod function_parameter;
#[path = "cli/governance_contract.rs"]
mod governance_contract;
#[path = "cli/help_contract.rs"]
mod help_contract;
#[path = "cli/impact_report.rs"]
mod impact_report;
#[path = "cli/incident_response_contract.rs"]
mod incident_response_contract;
#[path = "cli/inline_function/mod.rs"]
mod inline_function;
#[path = "cli/let_refactor/mod.rs"]
mod let_refactor;
#[path = "cli/maintainer_docs_contract.rs"]
mod maintainer_docs_contract;
#[path = "cli/msrv_docs_contract.rs"]
mod msrv_docs_contract;
#[path = "cli/package/mod.rs"]
mod package;
#[path = "cli/package_archive_contract.rs"]
mod package_archive_contract;
#[path = "cli/project_entrypoint_contract.rs"]
mod project_entrypoint_contract;
#[path = "cli/public_api_docs_contract.rs"]
mod public_api_docs_contract;
#[path = "cli/public_module_docs_contract.rs"]
mod public_module_docs_contract;
#[path = "cli/pull_request_template_contract.rs"]
mod pull_request_template_contract;
#[path = "cli/readme_api_docs_contract.rs"]
mod readme_api_docs_contract;
#[path = "cli/readme_ci_contract.rs"]
mod readme_ci_contract;
#[path = "cli/readme_contract.rs"]
mod readme_contract;
#[path = "cli/readme_docs_contract.rs"]
mod readme_docs_contract;
#[path = "cli/readme_install_contract.rs"]
mod readme_install_contract;
#[path = "cli/readme_smoke.rs"]
mod readme_smoke;
#[path = "cli/readme_workspace_smoke.rs"]
mod readme_workspace_smoke;
#[path = "cli/refactor_entrypoint_contract.rs"]
mod refactor_entrypoint_contract;
#[path = "cli/refactor_manifest/mod.rs"]
mod refactor_manifest;
#[path = "cli/refactor_preview.rs"]
mod refactor_preview;
#[path = "cli/refactor_workspace/mod.rs"]
mod refactor_workspace;
#[path = "cli/release_docs_contract.rs"]
mod release_docs_contract;
#[path = "cli/remove_unused_binding/mod.rs"]
mod remove_unused_binding;
#[path = "cli/rename/mod.rs"]
mod rename;
#[path = "cli/replace_forms.rs"]
mod replace_forms;
#[path = "cli/roadmap_contract.rs"]
mod roadmap_contract;
#[path = "cli/security_docs_contract.rs"]
mod security_docs_contract;
#[path = "cli/signature_report.rs"]
mod signature_report;
#[path = "cli/sort_definitions.rs"]
mod sort_definitions;
#[path = "cli/split_file.rs"]
mod split_file;
#[path = "cli/support_docs_contract.rs"]
mod support_docs_contract;
#[path = "cli/symbol_report.rs"]
mod symbol_report;
#[path = "cli/thread_expression/mod.rs"]
mod thread_expression;
#[path = "cli/unwrap_call.rs"]
mod unwrap_call;
#[path = "cli/workspace_entrypoint_contract.rs"]
mod workspace_entrypoint_contract;
#[path = "cli/workspace_report.rs"]
mod workspace_report;

fn paredit() -> Command {
    Command::cargo_bin("paredit").expect("binary")
}

fn cli_proptest_config(cases: u32) -> ProptestConfig {
    let mut config = ProptestConfig::with_cases(cases);
    config.failure_persistence = Some(Box::new(FileFailurePersistence::Off));
    config
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
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "paredit-cli-{name}-{}-{}-{timestamp}-{unique}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

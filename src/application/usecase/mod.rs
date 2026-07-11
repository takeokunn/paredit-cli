//! Application use cases for Lisp-aware analysis, reporting, and refactor planning.
//!
//! These services orchestrate typed domain operations into agent-facing plans,
//! reports, and workspace workflows without coupling to the CLI shell.

pub mod call_graph_report;
pub mod call_report;
pub(crate) mod callable_scope;
pub mod definition_report;
pub mod dependency_report;
pub mod duplicate_report;
pub mod extract_function;
pub mod form_report;
pub mod function_parameter;
pub mod impact_report;
pub mod inline_function;
pub mod inline_let;
pub mod introduce_let;
pub(crate) mod leading_trivia;
pub mod let_report;
pub mod package;
pub mod package_report;
pub mod remove_unused_binding;
pub mod remove_unused_definition;
pub mod rename;
pub mod replace_forms;
pub mod signature_report;
pub mod sort_definitions;
pub mod split_file;
pub mod thread_expression;
pub mod unthread_expression;
pub mod unwrap_call;
pub mod workspace_report;

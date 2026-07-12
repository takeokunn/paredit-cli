//! Core Lisp parsing, dialect, and semantic refactoring rules that stay
//! independent from CLI delivery and filesystem adapters.

pub mod call_graph_report;
pub mod common_lisp;
pub mod definition;
pub mod definition_report;
pub mod dialect;
pub mod form_shape;
pub mod form_similarity;
pub mod impact_report;
pub mod let_report;
pub mod lexical_scope;
pub mod refactor_plan;
pub mod refactor_preview;
pub mod rename;
pub mod report_policy;
pub mod sexpr;
pub mod signature_report;
pub mod similarity_report;

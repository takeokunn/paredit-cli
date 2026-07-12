//! Core Lisp parsing, dialect, and semantic refactoring rules that stay
//! independent from CLI delivery and filesystem adapters.

pub mod call_graph_report;
pub mod common_lisp;
pub mod definition;
pub mod dialect;
pub mod form_shape;
pub mod form_similarity;
pub mod lexical_scope;
pub mod report_policy;
pub mod sexpr;
pub mod similarity_report;

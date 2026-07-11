//! Core Lisp parsing, dialect, and semantic refactoring rules that stay
//! independent from CLI delivery and filesystem adapters.

pub mod common_lisp;
pub mod definition;
pub mod dialect;
pub mod lexical_scope;
pub mod sexpr;

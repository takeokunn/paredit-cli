//! Lexical binding and scope helpers shared by refactoring use cases.

mod bindings;
mod capture;
mod patterns;
mod syntax;
mod traversal;

#[cfg(test)]
mod tests;

pub use capture::value_capture;
pub use traversal::collect_unshadowed_symbol_references;

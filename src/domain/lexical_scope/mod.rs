//! Lexical binding and scope helpers shared by refactoring use cases.

mod bindings;
mod patterns;
mod syntax;
mod traversal;

#[cfg(test)]
mod tests;

pub use traversal::collect_unshadowed_symbol_references;

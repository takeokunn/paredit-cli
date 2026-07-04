//! Call-site inventory analysis.

mod collect;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

pub use collect::build_call_report;
pub use types::CallReportItem;

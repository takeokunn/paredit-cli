//! Signature inventory and arity compatibility analysis.

mod calls;
mod collect;
mod policy;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

pub use calls::classify_signature_call;
pub use collect::build_signature_reports;
pub use policy::evaluate_signature_report_policy;
pub use types::{
    SignatureCallItem, SignatureCallStatus, SignatureDefinitionItem, SignatureReportFile,
    SignatureReportPolicy, SignatureReportSource,
};

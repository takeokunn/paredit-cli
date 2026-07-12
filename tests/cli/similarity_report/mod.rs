pub(super) use super::*;
pub(super) use serde_json::Value;

fn parse_similarity_report(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).unwrap()
}

mod discovery;
mod error_policy;
mod help;
mod json;
mod limits;
mod scope;

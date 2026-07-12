use std::collections::BTreeMap;

use crate::application::usecase::call_report::CallReportItem;
use crate::application::usecase::signature_report::types::SignatureCallStatus;
use crate::domain::common_lisp::common_lisp_symbol_reference_needle;

/// `definitions_by_name` is keyed by `common_lisp_symbol_reference_needle`,
/// so all definitions whose names are `common_lisp_symbol_reference_eq`-equal
/// share one entry and each call classifies with one map lookup instead of a
/// linear scan over every definition.
pub fn classify_signature_call(
    definitions_by_name: &BTreeMap<String, Vec<(usize, Option<usize>)>>,
    call: &CallReportItem,
) -> (Option<(usize, Option<usize>)>, SignatureCallStatus) {
    let arities = definitions_by_name
        .get(&common_lisp_symbol_reference_needle(&call.head))
        .map(Vec::as_slice)
        .unwrap_or_default();
    let [(min, max)] = arities else {
        return if arities.is_empty() {
            (None, SignatureCallStatus::UnknownDefinition)
        } else {
            (None, SignatureCallStatus::AmbiguousDefinition)
        };
    };

    let status = if call.argument_count < *min {
        SignatureCallStatus::MissingArguments
    } else if max.is_some_and(|max| call.argument_count > max) {
        SignatureCallStatus::ExtraArguments
    } else {
        SignatureCallStatus::Exact
    };
    (Some((*min, *max)), status)
}

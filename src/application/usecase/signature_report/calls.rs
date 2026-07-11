use std::collections::BTreeMap;

use crate::application::usecase::call_report::CallReportItem;
use crate::application::usecase::signature_report::types::SignatureCallStatus;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;

pub fn classify_signature_call(
    definitions_by_name: &BTreeMap<String, Vec<(usize, Option<usize>)>>,
    call: &CallReportItem,
) -> (Option<(usize, Option<usize>)>, SignatureCallStatus) {
    let arities = definitions_by_name
        .iter()
        .filter(|(name, _)| common_lisp_symbol_reference_eq(name, &call.head))
        .flat_map(|(_, arities)| arities.iter().copied())
        .collect::<Vec<_>>();
    let [(min, max)] = arities.as_slice() else {
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

use std::collections::BTreeMap;

use crate::application::usecase::call_report::CallReportItem;
use crate::application::usecase::signature_report::types::SignatureCallStatus;
use crate::domain::common_lisp::common_lisp_symbol_name_eq;

pub fn classify_signature_call(
    definitions_by_name: &BTreeMap<String, Vec<usize>>,
    call: &CallReportItem,
) -> (Option<usize>, SignatureCallStatus) {
    let parameter_counts = definitions_by_name
        .iter()
        .filter(|(name, _)| common_lisp_symbol_name_eq(name, &call.head))
        .flat_map(|(_, counts)| counts.iter().copied())
        .collect::<Vec<_>>();
    let [expected] = parameter_counts.as_slice() else {
        return if parameter_counts.is_empty() {
            (None, SignatureCallStatus::UnknownDefinition)
        } else {
            (None, SignatureCallStatus::AmbiguousDefinition)
        };
    };

    let status = match call.argument_count.cmp(expected) {
        std::cmp::Ordering::Equal => SignatureCallStatus::Exact,
        std::cmp::Ordering::Less => SignatureCallStatus::MissingArguments,
        std::cmp::Ordering::Greater => SignatureCallStatus::ExtraArguments,
    };
    (Some(*expected), status)
}

use std::collections::BTreeMap;

use crate::application::usecase::call_report::CallReportItem;
use crate::application::usecase::signature_report::types::SignatureCallStatus;

pub fn classify_signature_call(
    definitions_by_name: &BTreeMap<String, Vec<usize>>,
    call: &CallReportItem,
) -> (Option<usize>, SignatureCallStatus) {
    let Some(parameter_counts) = definitions_by_name.get(&call.head) else {
        return (None, SignatureCallStatus::UnknownDefinition);
    };

    let [expected] = parameter_counts.as_slice() else {
        return (None, SignatureCallStatus::AmbiguousDefinition);
    };

    let status = match call.argument_count.cmp(expected) {
        std::cmp::Ordering::Equal => SignatureCallStatus::Exact,
        std::cmp::Ordering::Less => SignatureCallStatus::MissingArguments,
        std::cmp::Ordering::Greater => SignatureCallStatus::ExtraArguments,
    };
    (Some(*expected), status)
}

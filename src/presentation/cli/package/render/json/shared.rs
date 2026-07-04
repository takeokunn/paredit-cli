use crate::domain::sexpr::ByteSpan;

use serde_json::{json, Value};

pub(super) fn span_json(span: ByteSpan) -> Value {
    json!({
        "start": span.start().get(),
        "end": span.end().get(),
    })
}

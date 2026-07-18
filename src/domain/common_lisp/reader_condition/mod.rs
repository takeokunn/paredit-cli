//! Common Lisp reader-conditional dispatch detection.
//!
//! The S-expression parser intentionally represents `#+` and `#-` as atom
//! siblings of their feature expression and guarded datum. This module owns
//! the Common Lisp meaning of those dispatch atoms without changing that
//! general-purpose parser representation.

mod dispatch;
mod query;

#[cfg(test)]
pub use dispatch::CommonLispReaderConditionalDispatch;
pub use dispatch::{CommonLispReaderConditionalForm, CommonLispReaderConditionalKind};
#[cfg(test)]
pub use query::common_lisp_reader_conditional_dispatches;
pub use query::common_lisp_reader_conditional_forms;
pub(crate) use query::reader_conditional_kind;

//! Common Lisp reader-conditional dispatch detection.
//!
//! The S-expression parser intentionally represents `#+` and `#-` as atom
//! siblings of their feature expression and guarded datum. This module owns
//! the Common Lisp meaning of those dispatch atoms without changing that
//! general-purpose parser representation.

mod dispatch;
mod query;

pub use dispatch::{
    CommonLispReaderConditionalDispatch, CommonLispReaderConditionalForm,
    CommonLispReaderConditionalKind,
};
pub use query::{
    common_lisp_reader_conditional_dispatches, common_lisp_reader_conditional_forms,
    contains_common_lisp_reader_conditional,
};

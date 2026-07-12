//! Common Lisp reader-label dispatch detection.
//!
//! Reader labels (`#n=` and `#n#`) preserve object identity and can construct
//! cyclic objects. The general-purpose parser represents their dispatches as
//! atoms, so this module gives that syntax its Common Lisp meaning.

mod dispatch;
mod query;

pub use dispatch::{
    CommonLispReaderLabelDispatch, CommonLispReaderLabelForm, CommonLispReaderLabelKind,
};
pub use query::{common_lisp_reader_label_dispatches, common_lisp_reader_label_forms};

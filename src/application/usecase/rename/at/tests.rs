use super::{RenameAtError, plan_rename_at};
use crate::application::usecase::rename::{RenameAtNamespace, RenameAtRequest};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteOffset, SymbolName};

// NOT ported: `nested_lambda_initializers` (an outer binding renamed through
// a &optional/&key/&aux default-value initializer nested inside a local
// callable's or defmethod's lambda list, e.g. `(flet ((f (&optional (x
// outer))) ...))`) relies on `collect_enclosing_lambda_list_references`
// (rename/binding/lambda_like/lambda_list.rs on feat/macro-aware-refactoring),
// a substantial rewrite of this crate's shared lambda-list reference-collection
// walker that main does not have yet. Porting it changes behavior for every
// caller of `binding_rename_parts` (plain `rename-binding`, `introduce-let`),
// not just `rename-at`, so it needs its own reviewed change rather than
// riding along with this command's addition.
mod global;
mod local;
mod macro_namespaces;
mod safety;
mod validation;
mod value;

fn request<'a>(input: &'a str, needle: &str, to: &str) -> RenameAtRequest<'a> {
    RenameAtRequest {
        input,
        dialect: Dialect::CommonLisp,
        at: ByteOffset::new(input.find(needle).expect("needle")),
        to: SymbolName::new(to).expect("symbol"),
    }
}

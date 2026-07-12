mod body;
mod local_callables;
mod name_value;
mod variable_specs;

use anyhow::Result;

use crate::domain::common_lisp::{CommonLispBindingRefactorForm, CommonLispBindingReferenceScope};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionView, SymbolName};

use super::candidates::LetBindingRemovalCandidate;

use body::body_binding_reference_spans;
use local_callables::local_callable_binding_reference_spans;
use name_value::name_value_binding_reference_spans;
use variable_specs::variable_spec_binding_reference_spans;

pub(super) struct BindingReferenceContext<'a> {
    pub(super) dialect: Dialect,
    pub(super) input: &'a str,
    pub(super) target: &'a ExpressionView,
    pub(super) binding_form: &'a ExpressionView,
    pub(super) candidates: &'a [LetBindingRemovalCandidate],
    pub(super) candidate: &'a LetBindingRemovalCandidate,
    pub(super) name: &'a SymbolName,
}

#[expect(
    clippy::too_many_arguments,
    reason = "binding reference resolution takes the selected binding plus traversal context"
)]
pub(super) fn binding_reference_spans(
    dialect: Dialect,
    input: &str,
    target: &ExpressionView,
    refactor_form: CommonLispBindingRefactorForm,
    binding_form: &ExpressionView,
    candidates: &[LetBindingRemovalCandidate],
    candidate: &LetBindingRemovalCandidate,
    name: &SymbolName,
) -> Result<Vec<ByteSpan>> {
    let context = BindingReferenceContext {
        dialect,
        input,
        target,
        binding_form,
        candidates,
        candidate,
        name,
    };
    let Some(scope) = refactor_form.reference_scope() else {
        anyhow::bail!("remove-unused-binding unsupported reference scope");
    };

    match scope {
        CommonLispBindingReferenceScope::NameValuePairs(form) => {
            name_value_binding_reference_spans(
                &context,
                form.is_sequential() || binding_form.delimiter == Some(Delimiter::Bracket),
            )
        }
        CommonLispBindingReferenceScope::LocalCallableDefinitions(form) if !form.is_macro() => {
            local_callable_binding_reference_spans(dialect, target, name)
        }
        CommonLispBindingReferenceScope::LocalCallableDefinitions(_) => {
            Ok(body_binding_reference_spans(
                dialect,
                input,
                target,
                name,
                refactor_form.remove_unused_body_start_index(),
            ))
        }
        CommonLispBindingReferenceScope::VariableSpecs(spec_form, binding_form_kind) => Ok(
            variable_spec_binding_reference_spans(&context, spec_form, binding_form_kind),
        ),
        CommonLispBindingReferenceScope::BodyOnly => Ok(body_binding_reference_spans(
            dialect,
            input,
            target,
            name,
            refactor_form.remove_unused_body_start_index(),
        )),
    }
}

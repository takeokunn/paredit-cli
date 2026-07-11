//! Common Lisp syntax helpers shared across semantic refactoring passes.

mod forms;
mod operator;
mod scope;

pub(crate) use forms::{
    CommonLispBindingListShape, CommonLispBindingRefactorForm, CommonLispBindingReferenceScope,
    CommonLispHandlerBindingForm, CommonLispLambdaListShape, CommonLispLetBindingForm,
    CommonLispLocalCallableForm, CommonLispPackageDeclarationForm, CommonLispRuntimeDependencyForm,
    CommonLispSlotBindingForm, CommonLispValueScopeForm, CommonLispVariableBindingForm,
    CommonLispVariableSpecForm,
};
pub(crate) use operator::{
    CommonLispOperator, common_lisp_binding_refactor_form_for_head, common_lisp_operator_head_eq,
    common_lisp_symbol_name_eq, is_common_lisp_declaration_form,
    normalize_common_lisp_operator_head,
};
pub(crate) use scope::{
    common_lisp_local_callable_form, is_local_callable_bound, is_macro_callable_form,
    local_callable_binding_body_scope, local_callable_body_scope,
    local_callable_definition_reference_scope, local_callable_names, local_callable_scope_at_path,
};

#[cfg(test)]
mod tests;

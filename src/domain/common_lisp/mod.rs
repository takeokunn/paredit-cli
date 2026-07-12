//! Common Lisp syntax helpers shared across semantic refactoring passes.

mod forms;
mod function_value_namespace;
mod lambda_bindings;
mod operator;
mod reader_escape;
mod scope;

pub(crate) use forms::{
    CommonLispBindingListShape, CommonLispBindingRefactorForm, CommonLispBindingReferenceScope,
    CommonLispHandlerBindingForm, CommonLispLambdaListShape, CommonLispLetBindingForm,
    CommonLispLocalCallableForm, CommonLispPackageDeclarationForm, CommonLispRuntimeDependencyForm,
    CommonLispSlotBindingForm, CommonLispValueScopeForm, CommonLispVariableBindingForm,
    CommonLispVariableSpecForm,
};
pub(crate) use function_value_namespace::function_value_namespace_diagnostics;
pub(crate) use lambda_bindings::{
    destructuring_lambda_list_bindings, macro_lambda_list_bindings, ordinary_lambda_list_bindings,
};
pub(crate) use operator::{
    common_lisp_binding_refactor_form_for_head, common_lisp_operator_head_eq,
    common_lisp_symbol_name_eq, common_lisp_symbol_reference_eq, is_common_lisp_declaration_form,
    is_common_lisp_earmuffed_special_variable_name, normalize_common_lisp_operator_head,
    CommonLispOperator,
};
pub(crate) use reader_escape::common_lisp_reader_escape_diagnostics;
pub(crate) use scope::{
    common_lisp_local_callable_form, common_lisp_macro_expander_path, is_local_callable_bound,
    is_macro_callable_form, local_callable_binding_body_scope, local_callable_body_scope,
    local_callable_definition_reference_scope, local_callable_names, local_callable_scope_at_path,
};

#[cfg(test)]
mod tests;

//! Common Lisp syntax helpers shared across semantic refactoring passes.

mod forms;
mod operator;
mod reader_condition;
mod reader_label;
mod reader_literal;
mod scope;
mod special_binding;

pub(crate) use forms::{
    CommonLispBindingListShape, CommonLispBindingRefactorForm, CommonLispBindingReferenceScope,
    CommonLispDeclarationScope, CommonLispHandlerBindingForm, CommonLispLambdaListShape,
    CommonLispLetBindingForm, CommonLispLocalCallableForm, CommonLispPackageDeclarationForm,
    CommonLispResourceBindingForm, CommonLispRuntimeDependencyForm, CommonLispSlotBindingForm,
    CommonLispValueScopeForm, CommonLispVariableBindingForm, CommonLispVariableSpecForm,
};
pub(crate) use operator::{
    common_lisp_binding_refactor_form_for_head, common_lisp_operator_head_eq,
    common_lisp_symbol_name_eq, common_lisp_symbol_reference_eq,
    common_lisp_symbol_reference_needle, is_common_lisp_declaration_form,
    is_common_lisp_earmuffed_special_variable_name, normalize_common_lisp_operator_head,
    CommonLispOperator,
};
pub(crate) use reader_condition::{
    common_lisp_reader_conditional_dispatches, common_lisp_reader_conditional_forms,
    CommonLispReaderConditionalKind,
};
pub(crate) use reader_label::{
    common_lisp_reader_label_dispatches, common_lisp_reader_label_forms, CommonLispReaderLabelKind,
};
pub(crate) use reader_literal::{common_lisp_reader_literals, CommonLispReaderLiteralKind};
pub(crate) use scope::{
    common_lisp_local_callable_form, common_lisp_macro_expander_path, is_local_callable_bound,
    is_macro_callable_form, local_callable_binding_body_scope, local_callable_body_scope,
    local_callable_definition_reference_scope, local_callable_names, local_callable_scope_at_path,
};
pub(crate) use special_binding::{
    common_lisp_dynamic_binding_is_declared, common_lisp_special_declaration_body_start,
};

#[cfg(test)]
mod tests;

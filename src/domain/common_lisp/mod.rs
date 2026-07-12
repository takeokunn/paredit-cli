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
    CommonLispOperator, common_lisp_binding_refactor_form_for_head, common_lisp_operator_head_eq,
    common_lisp_symbol_name_eq, common_lisp_symbol_reference_eq, is_common_lisp_declaration_form,
    is_common_lisp_earmuffed_special_variable_name, normalize_common_lisp_operator_head,
};
pub(crate) use reader_condition::{
    CommonLispReaderConditionalDispatch, CommonLispReaderConditionalForm,
    CommonLispReaderConditionalKind, common_lisp_reader_conditional_dispatches,
    common_lisp_reader_conditional_forms, contains_common_lisp_reader_conditional,
};
pub(crate) use reader_label::{
    CommonLispReaderLabelDispatch, CommonLispReaderLabelForm, CommonLispReaderLabelKind,
    common_lisp_reader_label_dispatches, common_lisp_reader_label_forms,
};
pub(crate) use reader_literal::{
    CommonLispReaderLiteral, CommonLispReaderLiteralKind, common_lisp_reader_literals,
};
pub(crate) use scope::{
    ReaderExecutionContext, common_lisp_local_callable_form, common_lisp_macro_expander_path,
    is_local_callable_bound, is_macro_callable_form, local_callable_binding_body_scope,
    local_callable_body_scope, local_callable_definition_reference_scope, local_callable_names,
    local_callable_scope_at_path, reader_execution_context_at_path,
};
pub(crate) use special_binding::{
    common_lisp_dynamic_binding_is_declared, common_lisp_special_declaration_body_start,
};

#[cfg(test)]
mod tests;

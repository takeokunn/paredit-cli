use crate::domain::definition::DefinitionCategory;

use super::{
    CommonLispBindingRefactorForm, CommonLispHandlerBindingForm, CommonLispLambdaListShape,
    CommonLispLetBindingForm, CommonLispLocalCallableForm, CommonLispPackageDeclarationForm,
    CommonLispRuntimeDependencyForm, CommonLispSlotBindingForm, CommonLispValueScopeForm,
    CommonLispVariableBindingForm,
};

mod classify;
mod normalize;
mod table;

pub(crate) use normalize::{
    common_lisp_operator_head_eq, common_lisp_symbol_name_eq, common_lisp_symbol_reference_eq,
    is_common_lisp_declaration_form, is_common_lisp_earmuffed_special_variable_name,
    normalize_common_lisp_operator_head,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommonLispOperator {
    Let,
    LetStar,
    SymbolMacrolet,
    DestructuringBind,
    MultipleValueBind,
    HandlerCase,
    RestartCase,
    HandlerBind,
    RestartBind,
    Dolist,
    Dotimes,
    Do,
    DoStar,
    Prog,
    ProgStar,
    WithSlots,
    WithAccessors,
    Lambda,
    Fn,
    Defun,
    Defmacro,
    Defgeneric,
    DefineMethodCombination,
    Defmethod,
    ClDefmethod,
    Defclass,
    Defstruct,
    Deftype,
    DefineCondition,
    Defvar,
    Defglobal,
    DefineSymbolMacro,
    Defconstant,
    Defparameter,
    Defpackage,
    InPackage,
    Provide,
    Require,
    Load,
    LoadFile,
    LoadLibrary,
    UsePackage,
    Import,
    Defsystem,
    DefineSetfExpander,
    DefineCompilerMacro,
    DefineModifyMacro,
    Defsetf,
    Locally,
    Flet,
    Labels,
    Macrolet,
    CompilerMacrolet,
    Loop,
}

impl CommonLispOperator {
    pub(crate) fn from_head(head: &str) -> Option<Self> {
        table::common_lisp_operator_from_head(head)
    }

    pub(crate) fn is_parallel_let_binding(self) -> bool {
        matches!(self, Self::Let | Self::SymbolMacrolet)
    }

    pub(crate) fn is_sequential_let_binding(self) -> bool {
        self == Self::LetStar
    }

    pub(crate) fn is_let_binding(self) -> bool {
        self.is_parallel_let_binding() || self.is_sequential_let_binding()
    }

    pub(crate) fn let_binding_form(self) -> Option<CommonLispLetBindingForm> {
        match self {
            Self::Let => Some(CommonLispLetBindingForm::Parallel),
            Self::LetStar => Some(CommonLispLetBindingForm::Sequential),
            Self::SymbolMacrolet => Some(CommonLispLetBindingForm::SymbolMacro),
            _ => None,
        }
    }

    pub(crate) fn is_value_binding(self) -> bool {
        matches!(self, Self::DestructuringBind | Self::MultipleValueBind)
    }

    pub(crate) fn is_clause_binding(self) -> bool {
        matches!(self, Self::HandlerCase | Self::RestartCase)
    }

    pub(crate) fn is_handler_bind_binding(self) -> bool {
        matches!(self, Self::HandlerBind | Self::RestartBind)
    }

    pub(crate) fn includes_restart_bind_options(self) -> bool {
        self == Self::RestartBind
    }

    pub(crate) fn handler_binding_form(self) -> Option<CommonLispHandlerBindingForm> {
        match self {
            Self::HandlerBind => Some(CommonLispHandlerBindingForm::Handler),
            Self::RestartBind => Some(CommonLispHandlerBindingForm::Restart),
            _ => None,
        }
    }

    pub(crate) fn is_iteration_binding(self) -> bool {
        matches!(self, Self::Dolist | Self::Dotimes)
    }

    pub(crate) fn is_do_binding(self) -> bool {
        matches!(self, Self::Do | Self::DoStar)
    }

    pub(crate) fn is_prog_binding(self) -> bool {
        matches!(self, Self::Prog | Self::ProgStar)
    }

    pub(crate) fn is_sequential_variable_binding(self) -> bool {
        matches!(self, Self::DoStar | Self::ProgStar)
    }

    pub(crate) fn variable_binding_form(self) -> Option<CommonLispVariableBindingForm> {
        match self {
            Self::Do | Self::Prog => Some(CommonLispVariableBindingForm::Parallel),
            Self::DoStar | Self::ProgStar => Some(CommonLispVariableBindingForm::Sequential),
            _ => None,
        }
    }

    pub(crate) fn has_variable_step_forms(self) -> bool {
        self.is_do_binding()
    }

    pub(crate) fn value_scope_form(self) -> Option<CommonLispValueScopeForm> {
        if let Some(form) = self.let_binding_form() {
            return Some(CommonLispValueScopeForm::Let(form));
        }
        if let Some(form) = self.variable_binding_form() {
            return Some(CommonLispValueScopeForm::Variable(form));
        }
        if let Some(form) = self.local_callable_form() {
            return Some(CommonLispValueScopeForm::LocalCallable(form));
        }
        if let Some(form) = self.handler_binding_form() {
            return Some(CommonLispValueScopeForm::Handler(form));
        }

        match self {
            Self::Lambda => Some(CommonLispValueScopeForm::Lambda),
            Self::Fn => Some(CommonLispValueScopeForm::FunctionLiteral),
            operator if operator.is_defun_like() => Some(CommonLispValueScopeForm::Definition),
            operator if operator.is_value_binding() => Some(CommonLispValueScopeForm::Value),
            operator if operator.is_clause_binding() => Some(CommonLispValueScopeForm::Clause),
            operator if operator.is_iteration_binding() => {
                Some(CommonLispValueScopeForm::Iteration)
            }
            operator if operator.is_slot_binding() => Some(CommonLispValueScopeForm::Slot),
            _ => None,
        }
    }

    pub(crate) fn is_slot_binding(self) -> bool {
        matches!(self, Self::WithSlots | Self::WithAccessors)
    }

    pub(crate) fn slot_binding_form(self) -> Option<CommonLispSlotBindingForm> {
        match self {
            Self::WithSlots => Some(CommonLispSlotBindingForm::WithSlots),
            Self::WithAccessors => Some(CommonLispSlotBindingForm::WithAccessors),
            _ => None,
        }
    }

    pub(crate) fn is_lambda_like(self) -> bool {
        matches!(self, Self::Lambda | Self::Fn)
    }

    pub(crate) fn is_defun_like(self) -> bool {
        matches!(
            self,
            Self::Defun
                | Self::Defmacro
                | Self::DefineMethodCombination
                | Self::DefineSetfExpander
                | Self::DefineCompilerMacro
                | Self::DefineModifyMacro
                | Self::Defsetf
        )
    }

    pub(crate) fn is_inline_function_definition(self) -> bool {
        matches!(
            self,
            Self::Defun | Self::Defmacro | Self::DefineCompilerMacro
        )
    }

    pub(crate) fn is_setf_expander_definition(self) -> bool {
        self == Self::DefineSetfExpander
    }

    pub(crate) fn is_macro_expander_definition(self) -> bool {
        matches!(self, Self::DefineSetfExpander | Self::DefineCompilerMacro)
    }

    pub(crate) fn is_symbol_macrolet(self) -> bool {
        self == Self::SymbolMacrolet
    }

    pub(crate) fn is_locally(self) -> bool {
        self == Self::Locally
    }

    pub(crate) fn is_defsetf(self) -> bool {
        self == Self::Defsetf
    }

    pub(crate) fn is_method_definition(self) -> bool {
        matches!(self, Self::Defmethod | Self::ClDefmethod)
    }

    pub(crate) fn supports_function_parameter_refactor(self) -> bool {
        matches!(
            self,
            Self::Defun
                | Self::Defmacro
                | Self::DefineMethodCombination
                | Self::DefineSetfExpander
                | Self::DefineCompilerMacro
                | Self::DefineModifyMacro
                | Self::Defsetf
                | Self::Defmethod
                | Self::ClDefmethod
                | Self::Defgeneric
        )
    }

    pub(crate) fn definition_category(self) -> Option<DefinitionCategory> {
        classify::definition_category(self)
    }

    pub(crate) fn definition_lambda_list_shape(self) -> Option<CommonLispLambdaListShape> {
        classify::definition_lambda_list_shape(self)
    }

    pub(crate) fn runtime_dependency_form(self) -> Option<CommonLispRuntimeDependencyForm> {
        classify::runtime_dependency_form(self)
    }

    pub(crate) fn package_declaration_form(self) -> Option<CommonLispPackageDeclarationForm> {
        classify::package_declaration_form(self)
    }

    pub(crate) fn is_asdf_system_definition(self) -> bool {
        self == Self::Defsystem
    }

    pub(crate) fn is_local_callable_binding(self) -> bool {
        matches!(
            self,
            Self::Flet | Self::Labels | Self::Macrolet | Self::CompilerMacrolet
        )
    }

    pub(crate) fn local_callable_form(self) -> Option<CommonLispLocalCallableForm> {
        classify::local_callable_form(self)
    }

    pub(crate) fn binding_refactor_form(self) -> Option<CommonLispBindingRefactorForm> {
        if let Some(form) = self.let_binding_form() {
            return Some(CommonLispBindingRefactorForm::Let(form));
        }
        if let Some(form) = self.local_callable_form() {
            return Some(CommonLispBindingRefactorForm::LocalCallable(form));
        }
        if let Some(form) = self.handler_binding_form() {
            return Some(CommonLispBindingRefactorForm::Handler(form));
        }
        if let Some(form) = self.slot_binding_form() {
            return Some(CommonLispBindingRefactorForm::Slot(form));
        }

        match self {
            Self::DestructuringBind | Self::MultipleValueBind => {
                Some(CommonLispBindingRefactorForm::Value)
            }
            operator if operator.is_lambda_like() => {
                Some(CommonLispBindingRefactorForm::LambdaLike)
            }
            operator if operator.is_method_definition() => {
                Some(CommonLispBindingRefactorForm::MethodDefinition)
            }
            operator if operator.is_defun_like() => {
                Some(CommonLispBindingRefactorForm::FunctionDefinition)
            }
            Self::HandlerCase | Self::RestartCase => Some(CommonLispBindingRefactorForm::Clause),
            Self::Dolist | Self::Dotimes => Some(CommonLispBindingRefactorForm::Iteration),
            Self::Loop => Some(CommonLispBindingRefactorForm::Loop),
            Self::Do | Self::DoStar => Some(CommonLispBindingRefactorForm::Do(
                self.variable_binding_form()?,
            )),
            Self::Prog | Self::ProgStar => Some(CommonLispBindingRefactorForm::Prog(
                self.variable_binding_form()?,
            )),
            _ => None,
        }
    }
}

pub(crate) fn common_lisp_binding_refactor_form_for_head(
    head: &str,
) -> Option<CommonLispBindingRefactorForm> {
    CommonLispOperator::from_head(head)?.binding_refactor_form()
}

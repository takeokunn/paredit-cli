//! Domain knowledge for Lisp-family definition forms.

use crate::domain::dialect::Dialect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DefinitionCategory {
    Function,
    Macro,
    GenericFunction,
    Method,
    Class,
    Struct,
    Condition,
    Variable,
    Constant,
    Parameter,
    Package,
    System,
    Test,
    Customization,
    Mode,
    Other,
}

impl DefinitionCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Macro => "macro",
            Self::GenericFunction => "generic-function",
            Self::Method => "method",
            Self::Class => "class",
            Self::Struct => "struct",
            Self::Condition => "condition",
            Self::Variable => "variable",
            Self::Constant => "constant",
            Self::Parameter => "parameter",
            Self::Package => "package",
            Self::System => "system",
            Self::Test => "test",
            Self::Customization => "customization",
            Self::Mode => "mode",
            Self::Other => "other",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "function" => Some(Self::Function),
            "macro" => Some(Self::Macro),
            "generic-function" => Some(Self::GenericFunction),
            "method" => Some(Self::Method),
            "class" => Some(Self::Class),
            "struct" => Some(Self::Struct),
            "condition" => Some(Self::Condition),
            "variable" => Some(Self::Variable),
            "constant" => Some(Self::Constant),
            "parameter" => Some(Self::Parameter),
            "package" => Some(Self::Package),
            "system" => Some(Self::System),
            "test" => Some(Self::Test),
            "customization" => Some(Self::Customization),
            "mode" => Some(Self::Mode),
            "other" => Some(Self::Other),
            _ => None,
        }
    }

    pub fn is_callable(self) -> bool {
        matches!(
            self,
            Self::Function | Self::Macro | Self::GenericFunction | Self::Method
        )
    }
}

pub fn classify_definition_head(dialect: Dialect, head: &str) -> Option<DefinitionCategory> {
    let normalized = head
        .trim_start_matches("cl:")
        .trim_start_matches("cl-user:");
    let category = match normalized {
        "defun" | "cl-defun" | "defsubst" | "definline" | "defn" | "defn-" => {
            DefinitionCategory::Function
        }
        "defmacro"
        | "cl-defmacro"
        | "define-compiler-macro"
        | "define-modify-macro"
        | "define-setf-expander"
        | "defsetf" => DefinitionCategory::Macro,
        "defgeneric" | "cl-defgeneric" => DefinitionCategory::GenericFunction,
        "defmethod" | "cl-defmethod" => DefinitionCategory::Method,
        "defclass" | "cl-defclass" => DefinitionCategory::Class,
        "defstruct" | "cl-defstruct" | "defrecord" | "deftype" => DefinitionCategory::Struct,
        "define-condition" => DefinitionCategory::Condition,
        "defvar" | "defglobal" | "def" | "setq-default" => DefinitionCategory::Variable,
        "defconstant" | "defconst" => DefinitionCategory::Constant,
        "defparameter" | "defcustom" => {
            if normalized == "defcustom" {
                DefinitionCategory::Customization
            } else {
                DefinitionCategory::Parameter
            }
        }
        "defpackage" | "in-package" | "provide" | "require" => DefinitionCategory::Package,
        "asdf:defsystem" | "defsystem" => DefinitionCategory::System,
        "deftest" | "define-test" | "ert-deftest" | "define-ert-test" => DefinitionCategory::Test,
        "defgroup" | "defface" => DefinitionCategory::Customization,
        "define-minor-mode" | "define-derived-mode" | "define-globalized-minor-mode" => {
            DefinitionCategory::Mode
        }
        _ if dialect.is_definition_head(head) => DefinitionCategory::Other,
        _ if normalized.starts_with("define-") => DefinitionCategory::Other,
        _ => return None,
    };

    Some(category)
}

pub fn definition_name_child_index(_head: &str) -> Option<usize> {
    Some(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_round_trip_to_categories() {
        let categories = [
            DefinitionCategory::Function,
            DefinitionCategory::Macro,
            DefinitionCategory::GenericFunction,
            DefinitionCategory::Method,
            DefinitionCategory::Class,
            DefinitionCategory::Struct,
            DefinitionCategory::Condition,
            DefinitionCategory::Variable,
            DefinitionCategory::Constant,
            DefinitionCategory::Parameter,
            DefinitionCategory::Package,
            DefinitionCategory::System,
            DefinitionCategory::Test,
            DefinitionCategory::Customization,
            DefinitionCategory::Mode,
            DefinitionCategory::Other,
        ];

        for category in categories {
            assert_eq!(
                DefinitionCategory::from_label(category.label()),
                Some(category)
            );
        }
        assert_eq!(DefinitionCategory::from_label("unknown"), None);
    }

    #[test]
    fn classifies_common_lisp_and_emacs_definition_heads() {
        assert_eq!(
            classify_definition_head(Dialect::CommonLisp, "defun"),
            Some(DefinitionCategory::Function)
        );
        assert_eq!(
            classify_definition_head(Dialect::CommonLisp, "cl:defmacro"),
            Some(DefinitionCategory::Macro)
        );
        assert_eq!(
            classify_definition_head(Dialect::CommonLisp, "define-setf-expander"),
            Some(DefinitionCategory::Macro)
        );
        assert_eq!(
            classify_definition_head(Dialect::EmacsLisp, "defcustom"),
            Some(DefinitionCategory::Customization)
        );
        assert_eq!(
            classify_definition_head(Dialect::EmacsLisp, "define-minor-mode"),
            Some(DefinitionCategory::Mode)
        );
    }

    #[test]
    fn classifies_clojure_and_custom_define_heads() {
        assert_eq!(
            classify_definition_head(Dialect::Clojure, "defn-"),
            Some(DefinitionCategory::Function)
        );
        assert_eq!(
            classify_definition_head(Dialect::Clojure, "defrecord"),
            Some(DefinitionCategory::Struct)
        );
        assert_eq!(
            classify_definition_head(Dialect::Unknown, "define-widget"),
            Some(DefinitionCategory::Other)
        );
        assert_eq!(classify_definition_head(Dialect::Unknown, "let"), None);
    }

    #[test]
    fn callable_categories_are_limited_to_invokable_definition_kinds() {
        assert!(DefinitionCategory::Function.is_callable());
        assert!(DefinitionCategory::Macro.is_callable());
        assert!(DefinitionCategory::GenericFunction.is_callable());
        assert!(DefinitionCategory::Method.is_callable());
        assert!(!DefinitionCategory::Class.is_callable());
        assert!(!DefinitionCategory::Variable.is_callable());
    }
}

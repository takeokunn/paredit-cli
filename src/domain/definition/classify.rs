use crate::domain::common_lisp::{CommonLispOperator, normalize_common_lisp_operator_head};
use crate::domain::dialect::Dialect;

use super::DefinitionCategory;

pub(super) fn classify_definition_head(dialect: Dialect, head: &str) -> Option<DefinitionCategory> {
    if matches!(dialect, Dialect::CommonLisp | Dialect::Unknown) {
        if let Some(category) =
            CommonLispOperator::from_head(head).and_then(CommonLispOperator::definition_category)
        {
            return Some(category);
        }
    }

    let normalized = normalize_common_lisp_operator_head(head);
    let normalized_lower = normalized.to_ascii_lowercase();
    let category = match normalized_lower.as_str() {
        "cl-defun" | "defsubst" | "definline" | "defn" | "defn-" => DefinitionCategory::Function,
        "cl-defmacro" => DefinitionCategory::Macro,
        "cl-defgeneric" => DefinitionCategory::GenericFunction,
        "cl-defmethod" => DefinitionCategory::Method,
        "cl-defclass" => DefinitionCategory::Class,
        "cl-defstruct" | "defrecord" => DefinitionCategory::Struct,
        "def" | "setq-default" => DefinitionCategory::Variable,
        "defconst" => DefinitionCategory::Constant,
        "defparameter" | "defcustom" => {
            if normalized_lower == "defcustom" {
                DefinitionCategory::Customization
            } else {
                DefinitionCategory::Parameter
            }
        }
        "deftest" | "define-test" | "ert-deftest" | "define-ert-test" => DefinitionCategory::Test,
        "provide" | "require" => DefinitionCategory::Package,
        "defgroup" | "defface" => DefinitionCategory::Customization,
        "define-minor-mode" | "define-derived-mode" | "define-globalized-minor-mode" => {
            DefinitionCategory::Mode
        }
        _ if dialect.is_definition_head(head) => DefinitionCategory::Other,
        _ if normalized_lower.starts_with("define-") => DefinitionCategory::UnknownMacro,
        _ => return None,
    };

    Some(category)
}

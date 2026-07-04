use std::fmt;
use std::path::Path;
use std::str::FromStr;

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    CommonLisp,
    EmacsLisp,
    Scheme,
    Clojure,
    Janet,
    Fennel,
    Unknown,
}

impl Dialect {
    pub fn detect(path: Option<&Path>, explicit: Option<Self>) -> Self {
        if let Some(dialect) = explicit {
            return dialect;
        }
        path.and_then(|path| path.extension().and_then(|extension| extension.to_str()))
            .map_or(Self::Unknown, Self::from_extension)
    }

    pub fn from_extension(extension: &str) -> Self {
        match extension {
            "lisp" | "lsp" | "cl" | "asd" => Self::CommonLisp,
            "el" => Self::EmacsLisp,
            "scm" | "ss" | "sld" => Self::Scheme,
            "clj" | "cljs" | "cljc" | "edn" => Self::Clojure,
            "janet" => Self::Janet,
            "fnl" => Self::Fennel,
            _ => Self::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::CommonLisp => "common-lisp",
            Self::EmacsLisp => "emacs-lisp",
            Self::Scheme => "scheme",
            Self::Clojure => "clojure",
            Self::Janet => "janet",
            Self::Fennel => "fennel",
            Self::Unknown => "unknown",
        }
    }

    pub fn family(self) -> &'static str {
        match self {
            Self::CommonLisp => "Common Lisp",
            Self::EmacsLisp => "Emacs Lisp",
            Self::Scheme => "Scheme",
            Self::Clojure => "Clojure",
            Self::Janet => "Janet",
            Self::Fennel => "Fennel",
            Self::Unknown => "generic s-expression",
        }
    }

    pub fn is_definition_head(self, head: &str) -> bool {
        match self {
            Self::CommonLisp => matches!(
                head,
                "defun"
                    | "defmacro"
                    | "defmethod"
                    | "defgeneric"
                    | "defclass"
                    | "defstruct"
                    | "defsetf"
                    | "define-modify-macro"
                    | "define-setf-expander"
                    | "defpackage"
                    | "in-package"
                    | "defparameter"
                    | "defvar"
                    | "defconstant"
                    | "defsystem"
                    | "asdf:defsystem"
            ),
            Self::EmacsLisp => matches!(
                head,
                "defun"
                    | "defmacro"
                    | "defsubst"
                    | "defvar"
                    | "defconst"
                    | "defcustom"
                    | "defgroup"
                    | "define-minor-mode"
                    | "define-derived-mode"
                    | "provide"
                    | "require"
            ),
            Self::Scheme => matches!(
                head,
                "define" | "define-syntax" | "define-library" | "lambda" | "let" | "let*"
            ),
            Self::Clojure => matches!(
                head,
                "ns" | "def"
                    | "defn"
                    | "defmacro"
                    | "defrecord"
                    | "deftype"
                    | "defprotocol"
                    | "defmulti"
                    | "defmethod"
            ),
            Self::Janet => matches!(head, "def" | "defn" | "defmacro" | "def-" | "defn-"),
            Self::Fennel => matches!(head, "fn" | "lambda" | "macro" | "local" | "global"),
            Self::Unknown => head.starts_with("def") || matches!(head, "define" | "ns"),
        }
    }
}

impl FromStr for Dialect {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" | "unknown" => Ok(Self::Unknown),
            "common-lisp" | "cl" => Ok(Self::CommonLisp),
            "emacs-lisp" | "elisp" | "el" => Ok(Self::EmacsLisp),
            "scheme" | "scm" => Ok(Self::Scheme),
            "clojure" | "clj" => Ok(Self::Clojure),
            "janet" => Ok(Self::Janet),
            "fennel" | "fnl" => Ok(Self::Fennel),
            _ => Err(anyhow!("unsupported dialect: {s}")),
        }
    }
}

impl fmt::Display for Dialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_lisp_extensions() {
        assert_eq!(Dialect::from_extension("lisp"), Dialect::CommonLisp);
        assert_eq!(Dialect::from_extension("asd"), Dialect::CommonLisp);
    }

    #[test]
    fn detects_emacs_lisp_extension() {
        assert_eq!(Dialect::from_extension("el"), Dialect::EmacsLisp);
    }
}

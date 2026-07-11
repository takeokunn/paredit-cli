//! Dialect detection and capability helpers for Lisp-family files, including
//! extension-based routing and parser-affecting feature checks.

mod capability;
mod parse;

#[cfg(test)]
mod tests;

use std::path::Path;

/// Selects Lisp-family parsing and refactoring rules for a source file.
///
/// # Examples
///
/// ```
/// use paredit_cli::dialect::Dialect;
///
/// assert_eq!(Dialect::from_extension("el"), Dialect::EmacsLisp);
/// assert_eq!(
///     Dialect::detect(Some(std::path::Path::new("init.el")), None),
///     Dialect::EmacsLisp
/// );
/// ```
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
    /// Resolves the effective dialect from an explicit override or file extension.
    pub fn detect(path: Option<&Path>, explicit: Option<Self>) -> Self {
        if let Some(dialect) = explicit {
            return dialect;
        }
        path.and_then(|path| path.extension().and_then(|extension| extension.to_str()))
            .map_or(Self::Unknown, Self::from_extension)
    }

    /// Maps a lowercase file extension onto the closest supported dialect.
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

    /// Returns the stable CLI and JSON identifier for this dialect.
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

    /// Returns a human-facing family label for diagnostics and reports.
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
}

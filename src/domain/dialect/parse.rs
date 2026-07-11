use std::fmt;
use std::str::FromStr;

use anyhow::anyhow;

use super::Dialect;

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

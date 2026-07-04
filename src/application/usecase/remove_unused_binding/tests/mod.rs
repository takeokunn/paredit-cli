use proptest::{prelude::*, test_runner::TestCaseError};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName, SyntaxTree};

use super::{RemoveUnusedBindingRequest, plan_remove_unused_binding};

mod clojure;
mod common_lisp;
mod pbt;

fn target(input: &str) -> ExpressionView {
    let tree = SyntaxTree::parse(input).expect("parse");
    tree.select_path(&"0".parse::<Path>().expect("path"))
        .expect("select")
        .view()
}

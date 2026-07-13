use proptest::{prelude::*, test_runner::TestCaseError};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName, SyntaxTree};

use super::{RemoveUnusedBindingPlan, RemoveUnusedBindingRequest, plan_remove_unused_binding};

mod clojure;
mod common_lisp;
mod pbt;

fn symbol(name: &str) -> SymbolName {
    SymbolName::new(name).expect("symbol")
}

fn target(input: &str) -> ExpressionView {
    target_at(input, "0")
}

fn target_at(input: &str, path: &str) -> ExpressionView {
    let tree = SyntaxTree::parse(input).expect("parse");
    tree.select_path(&path.parse::<Path>().expect("path"))
        .expect("select")
        .view()
}

fn plan_remove_unused_binding_for(
    input: &str,
    dialect: Dialect,
    path: Option<&str>,
    name: Option<&str>,
    all_bindings: bool,
    allow_drop_value: bool,
) -> RemoveUnusedBindingPlan {
    let parsed_path = path.map(|path| path.parse().expect("path"));
    let symbol = name.map(symbol);
    plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect,
        path: parsed_path,
        target: target_at(input, path.unwrap_or("0")),
        name: symbol.as_ref(),
        all_bindings,
        allow_drop_value,
    })
    .expect("plan")
}

fn remove_unused_binding_error(
    input: &str,
    dialect: Dialect,
    name: Option<&str>,
    all_bindings: bool,
    allow_drop_value: bool,
) -> String {
    let symbol = name.map(symbol);
    plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect,
        path: None,
        target: target(input),
        name: symbol.as_ref(),
        all_bindings,
        allow_drop_value,
    })
    .expect_err("expected remove-unused-binding to fail")
    .to_string()
}

fn remove_unused_binding_error_for(
    input: &str,
    dialect: Dialect,
    path: Option<&str>,
    name: Option<&str>,
    all_bindings: bool,
    allow_drop_value: bool,
) -> String {
    let parsed_path = path.map(|path| path.parse().expect("path"));
    let symbol = name.map(symbol);
    plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect,
        path: parsed_path,
        target: target_at(input, path.unwrap_or("0")),
        name: symbol.as_ref(),
        all_bindings,
        allow_drop_value,
    })
    .expect_err("expected remove-unused-binding to fail")
    .to_string()
}

fn common_lisp_plan(
    input: &str,
    name: Option<&str>,
    all_bindings: bool,
    allow_drop_value: bool,
) -> RemoveUnusedBindingPlan {
    plan_remove_unused_binding_for(
        input,
        Dialect::CommonLisp,
        None,
        name,
        all_bindings,
        allow_drop_value,
    )
}

fn common_lisp_error(
    input: &str,
    name: Option<&str>,
    all_bindings: bool,
    allow_drop_value: bool,
) -> String {
    remove_unused_binding_error(
        input,
        Dialect::CommonLisp,
        name,
        all_bindings,
        allow_drop_value,
    )
}

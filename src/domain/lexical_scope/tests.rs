use proptest::prelude::*;

use super::*;
use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};

fn selected_form(input: &str) -> crate::domain::sexpr::ExpressionView {
    let tree = SyntaxTree::parse(input).expect("parse");
    tree.select_path(&"0".parse::<Path>().expect("path"))
        .expect("select")
        .view()
}

fn reference_texts(input: &str, symbol: &str) -> Vec<String> {
    let view = selected_form(input);
    let symbol = SymbolName::new(symbol).expect("symbol");
    let mut spans = Vec::new();
    collect_unshadowed_symbol_references(&view, &symbol, input, &mut spans);
    spans
        .into_iter()
        .map(|span| span.slice(input).to_owned())
        .collect()
}

#[test]
fn skips_shadowed_lambda_parameter_references() {
    let input = "(list x (lambda (x) x))";

    assert_eq!(reference_texts(input, "x"), vec!["x"]);
}

#[test]
fn sequential_let_stops_after_shadowing_binding() {
    let input = "(let* ((y x) (x 2)) (list x y))";

    assert_eq!(reference_texts(input, "x"), vec!["x"]);
}

#[test]
fn parallel_let_checks_binding_values_before_body_shadowing() {
    let input = "(let ((x 1) (y x)) (list x y))";

    assert_eq!(reference_texts(input, "x"), vec!["x"]);
}

#[test]
fn symbol_macrolet_checks_expansions_before_body_shadowing() {
    let input = "(symbol-macrolet ((x outer) (y x)) (list x y outer))";

    assert_eq!(reference_texts(input, "x"), vec!["x"]);
    assert_eq!(reference_texts(input, "outer"), vec!["outer", "outer"]);
}

#[test]
fn clojure_vector_let_is_sequential_for_shadowing() {
    let input = "(let [y x x 2] (list x y))";

    assert_eq!(reference_texts(input, "x"), vec!["x"]);
}

#[test]
fn clojure_destructuring_shadows_keys_shorthand() {
    let input = "(list x (fn [{:keys [x] :as m}] x m))";

    assert_eq!(reference_texts(input, "x"), vec!["x"]);
}

#[test]
fn lambda_list_default_forms_remain_outer_references() {
    let input = "(list fallback (lambda (&optional (x (fallback y) supplied)) x))";

    assert_eq!(
        reference_texts(input, "fallback"),
        vec!["fallback", "fallback"]
    );
}

#[test]
fn define_setf_expander_body_is_definition_scope_boundary() {
    let input = "(list outer (define-setf-expander slot (place) (list outer place)) outer)";

    assert_eq!(reference_texts(input, "outer"), vec!["outer", "outer"]);
}

#[test]
fn define_compiler_macro_body_is_definition_scope_boundary() {
    let input = "(list outer (define-compiler-macro render (place) (list outer place)) outer)";

    assert_eq!(reference_texts(input, "outer"), vec!["outer", "outer"]);
}

proptest! {
    #[test]
    fn pbt_shadowed_lambda_references_are_not_counted(count in 1usize..12) {
        let lambdas = std::iter::repeat_n("(lambda (x) x)", count)
            .collect::<Vec<_>>()
            .join(" ");
        let input = format!("(list x {lambdas})");

        prop_assert_eq!(reference_texts(&input, "x"), vec!["x"]);
    }

    #[test]
    fn pbt_sequential_let_counts_values_before_shadowing(count in 1usize..12) {
        let earlier_bindings = (0..count)
            .map(|index| format!("(y{index} x)"))
            .collect::<Vec<_>>()
            .join(" ");
        let input = format!("(let* ({earlier_bindings} (x 2)) (list x))");

        prop_assert_eq!(reference_texts(&input, "x").len(), count);
    }

    #[test]
    fn pbt_clojure_vector_let_counts_values_before_shadowing(count in 1usize..12) {
        let earlier_bindings = (0..count)
            .map(|index| format!("y{index} x"))
            .collect::<Vec<_>>()
            .join(" ");
        let input = format!("(let [{earlier_bindings} x 2] (list x))");

        prop_assert_eq!(reference_texts(&input, "x").len(), count);
    }
}

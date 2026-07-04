use proptest::prelude::*;

use super::*;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};

fn infer_at(input: &str, path: &[usize], explicit: &[&str]) -> Vec<String> {
    let tree = SyntaxTree::parse(input).expect("parse fixture");
    let selection = tree
        .select_path(&Path::from_indexes(path.to_vec()))
        .expect("select fixture");
    let explicit = explicit
        .iter()
        .map(|param| (*param).to_owned())
        .collect::<Vec<_>>();

    infer_extract_function_params(&selection.view(), &explicit)
}

fn plan_at(
    input: &str,
    path: &[usize],
    name: &str,
    explicit: &[&str],
    infer_params: bool,
) -> ExtractFunctionPlan {
    let tree = SyntaxTree::parse(input).expect("parse fixture");
    let selection = tree
        .select_path(&Path::from_indexes(path.to_vec()))
        .expect("select fixture");
    let explicit_params = explicit
        .iter()
        .map(|param| (*param).to_owned())
        .collect::<Vec<_>>();

    plan_extract_function(ExtractFunctionRequest {
        input,
        selection,
        path: Some(Path::from_indexes(path.to_vec())),
        dialect: Dialect::CommonLisp,
        name: SymbolName::new(name).expect("symbol fixture"),
        explicit_params,
        infer_params,
        insert: ExtractFunctionInsert::Append,
        anchor_path: None,
    })
    .expect("plan extract function")
}

#[test]
fn infers_free_variables_from_selected_expression() {
    let params = infer_at(
        "(defun render (width height margin) (+ (* width height) margin))",
        &[0, 3],
        &[],
    );

    assert_eq!(params, vec!["width", "height", "margin"]);
}

#[test]
fn excludes_local_let_bindings_from_body() {
    let params = infer_at("(let ((local input)) (+ local outer))", &[0], &[]);

    assert_eq!(params, vec!["input", "outer"]);
}

#[test]
fn treats_let_star_bindings_as_sequential() {
    let params = infer_at(
        "(let* ((first input) (second first)) (+ first second outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["input", "outer"]);
}

#[test]
fn excludes_destructured_lambda_parameters_and_explicit_params() {
    let params = infer_at(
        "(lambda [{:keys [inner]}] (+ inner outer ignored))",
        &[0],
        &["ignored"],
    );

    assert_eq!(params, vec!["outer"]);
}

#[test]
fn plans_extract_function_with_inferred_params() {
    let plan = plan_at(
        "(defun render () (+ width height))",
        &[0, 3],
        "area",
        &[],
        true,
    );

    assert_eq!(plan.call, "(area width height)");
    assert_eq!(
        plan.definition,
        "(defun area (width height) (+ width height))"
    );
    assert_eq!(plan.inferred_params, vec!["width", "height"]);
    assert!(plan.changed);
    SyntaxTree::parse(&plan.rewritten).expect("rewritten output remains parseable");
}

#[test]
fn appends_extracted_definition_after_blank_line_when_input_has_trailing_newline() {
    let plan = plan_at(
        "(defun render () (+ 1 2))\n",
        &[0, 3],
        "render-sum",
        &[],
        false,
    );

    assert_eq!(
        plan.rewritten,
        "(defun render () (render-sum))\n\n(defun render-sum () (+ 1 2))\n"
    );
}

#[test]
fn plans_extract_function_before_anchor() {
    let input = "(defun first () value)\n(defun second () (+ x y))\n";
    let tree = SyntaxTree::parse(input).expect("parse fixture");
    let selection = tree
        .select_path(&Path::from_indexes(vec![1, 3]))
        .expect("select fixture");

    let plan = plan_extract_function(ExtractFunctionRequest {
        input,
        selection,
        path: Some(Path::from_indexes(vec![1, 3])),
        dialect: Dialect::CommonLisp,
        name: SymbolName::new("sum").expect("symbol fixture"),
        explicit_params: Vec::new(),
        infer_params: true,
        insert: ExtractFunctionInsert::Before,
        anchor_path: Some(Path::from_indexes(vec![0])),
    })
    .expect("plan extract function");

    assert!(plan.rewritten.starts_with("(defun sum (x y) (+ x y))\n\n"));
    assert!(plan.anchor_span.is_some());
}

fn symbol_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,8}".prop_filter("reserved symbol", |name| {
        !matches!(
            name.as_str(),
            "let"
                | "let*"
                | "lambda"
                | "fn"
                | "defun"
                | "defmacro"
                | "nil"
                | "t"
                | "true"
                | "false"
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn pbt_infers_each_free_symbol_once_and_excludes_bound_symbols(
        outer_a in symbol_strategy(),
        outer_b in symbol_strategy(),
        local in symbol_strategy(),
    ) {
        prop_assume!(outer_a != outer_b);
        prop_assume!(outer_a != local);
        prop_assume!(outer_b != local);

        let input = format!("(+ {outer_a} {outer_b} (let (({local} {outer_a})) (+ {local} {outer_b})))");
        let tree = SyntaxTree::parse(&input).expect("parse generated input");
        let selection = tree
            .select_path(&Path::from_indexes(vec![0]))
            .expect("select generated input");

        let inferred = infer_extract_function_params(&selection.view(), &[]);

        prop_assert_eq!(inferred, vec![outer_a, outer_b]);
    }

    #[test]
    fn pbt_explicit_params_are_not_inferred_again(
        outer_a in symbol_strategy(),
        outer_b in symbol_strategy(),
    ) {
        prop_assume!(outer_a != outer_b);

        let input = format!("(+ {outer_a} {outer_b} {outer_a})");
        let tree = SyntaxTree::parse(&input).expect("parse generated input");
        let selection = tree
            .select_path(&Path::from_indexes(vec![0]))
            .expect("select generated input");
        let explicit = vec![outer_a];

        let inferred = infer_extract_function_params(&selection.view(), &explicit);

        prop_assert_eq!(inferred, vec![outer_b]);
    }

    #[test]
    fn pbt_planned_extract_function_output_remains_parseable(
        outer_a in symbol_strategy(),
        outer_b in symbol_strategy(),
    ) {
        prop_assume!(outer_a != outer_b);

        let input = format!("(defun source () (+ {outer_a} {outer_b}))");
        let tree = SyntaxTree::parse(&input).expect("parse generated input");
        let selection = tree
            .select_path(&Path::from_indexes(vec![0, 3]))
            .expect("select generated input");

        let plan = plan_extract_function(ExtractFunctionRequest {
            input: &input,
            selection,
            path: Some(Path::from_indexes(vec![0, 3])),
            dialect: Dialect::CommonLisp,
            name: SymbolName::new("extracted").expect("symbol fixture"),
            explicit_params: Vec::new(),
            infer_params: true,
            insert: ExtractFunctionInsert::Append,
            anchor_path: None,
        })
        .expect("plan generated extraction");

        prop_assert!(SyntaxTree::parse(&plan.rewritten).is_ok());
        prop_assert_eq!(plan.params, vec![outer_a, outer_b]);
    }
}

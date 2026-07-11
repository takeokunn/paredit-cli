use super::*;

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

#[test]
fn rejects_relative_extract_function_insertion_without_anchor_path() {
    let input = "(defun render () (+ x y))\n";
    let tree = SyntaxTree::parse(input).expect("parse fixture");
    let selection = tree
        .select_path(&Path::from_indexes(vec![0, 3]))
        .expect("select fixture");

    let error = plan_extract_function(ExtractFunctionRequest {
        input,
        selection,
        path: Some(Path::from_indexes(vec![0, 3])),
        dialect: Dialect::CommonLisp,
        name: SymbolName::new("sum").expect("symbol fixture"),
        explicit_params: Vec::new(),
        infer_params: true,
        insert: ExtractFunctionInsert::Before,
        anchor_path: None,
    })
    .expect_err("missing anchor path should be rejected");

    assert_eq!(
        error.to_string(),
        "--insert before/after requires --anchor-path"
    );
}

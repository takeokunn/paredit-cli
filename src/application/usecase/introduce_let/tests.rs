use super::*;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};
use proptest::prelude::*;

fn request<'a>(input: &'a str, path: &str, all_occurrences: bool) -> IntroduceLetRequest<'a> {
    let tree = SyntaxTree::parse(input).expect("parse");
    let path = path.parse::<Path>().expect("path");
    let selection = tree.select_path(&path).expect("select");
    IntroduceLetRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: Some(path),
        target: selection.view(),
        enclosing_span: selection.enclosing_list_span().expect("enclosing"),
        name: SymbolName::new("product").expect("symbol"),
        all_occurrences,
    }
}

#[test]
fn introduces_single_selected_occurrence_by_default() {
    let input = "(defun render () (+ (* width height) margin (* width height)))";
    let plan = plan_introduce_let(request(input, "0.3.1", false)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product margin (* width height))))"
    );
}

#[test]
fn introduces_all_structurally_equivalent_occurrences() {
    let input = "(defun render () (+ (* width height) margin (*  width height)))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 2);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 0);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product margin product)))"
    );
}

#[test]
fn skips_all_occurrences_inside_shadowing_binding_forms() {
    let input = "(defun render () (+ (* width height) (let ((product 1)) (* width height))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 1);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (let ((product 1)) (* width height)))))"
    );
}

#[test]
fn skips_all_occurrences_inside_symbol_macrolet_shadowing_binding_forms() {
    let input =
        "(defun render () (+ (* width height) (symbol-macrolet ((product 1)) (* width height))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 1);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (symbol-macrolet ((product 1)) (* width height)))))"
    );
}

#[test]
fn keeps_let_star_same_binding_initializer_in_outer_scope() {
    let input = "(defun render () (+ (* width height) (let* ((product (* width height))) (* width height))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 2);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (let* ((product product)) (* width height)))))"
    );
}

#[test]
fn skips_let_star_later_initializer_shadowed_by_previous_binding() {
    let input = "(defun render () (+ (* width height) (let* ((product 1) (other (* width height))) (* width height))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 1);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 2);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (let* ((product 1) (other (* width height))) (* width height)))))"
    );
}

#[test]
fn skips_all_occurrences_inside_define_setf_expander_shadowing_lambda_list() {
    let input = "(defun render () (+ (* width height) (define-setf-expander slot (&environment product place) (* width height))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 1);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (define-setf-expander slot (&environment product place) (* width height)))))"
    );
}

#[test]
fn skips_all_occurrences_inside_define_compiler_macro_shadowing_lambda_list() {
    let input = "(defun render () (+ (* width height) (define-compiler-macro slot (&environment product place) (* width height))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 1);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (define-compiler-macro slot (&environment product place) (* width height)))))"
    );
}

#[test]
fn skips_all_occurrences_inside_destructuring_bind_shadowing_body_only() {
    let input = "(defun render () (+ (* width height) (destructuring-bind (product) (* width height) (* width height))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 2);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (destructuring-bind (product) product (* width height)))))"
    );
}

#[test]
fn skips_all_occurrences_inside_handler_case_shadowing_clause_only() {
    let input = "(defun render () (+ (* width height) (handler-case (* width height) (error (product) (* width height)) (:no-error (value) (* width height)))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 3);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (handler-case product (error (product) (* width height)) (:no-error (value) product)))))"
    );
}

#[test]
fn skips_all_occurrences_inside_macrolet_lambda_body_only() {
    let input = "(defun render () (+ (* width height) (macrolet ((with-product (product) (* width height))) (* width height))))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 2);
    assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (macrolet ((with-product (product) (* width height))) product))))"
    );
}

#[test]
fn rejects_selected_expression_inside_shadowing_binding_form() {
    let input = "(defun render () (let ((product 1)) (* width height)))";
    let error = plan_introduce_let(request(input, "0.3.2", false)).expect_err("shadowed");

    assert!(
        error
            .to_string()
            .contains("inside an existing binding for 'product'")
    );
}

#[test]
fn rejects_selected_expression_inside_symbol_macrolet_shadowing_binding_form() {
    let input = "(defun render () (symbol-macrolet ((product 1)) (* width height)))";
    let error = plan_introduce_let(request(input, "0.3.2", false)).expect_err("shadowed");

    assert!(
        error
            .to_string()
            .contains("inside an existing binding for 'product'")
    );
}

#[test]
fn rejects_selected_expression_inside_define_setf_expander_shadowing_lambda_list() {
    let input = "(defun render () (define-setf-expander slot (&environment product place) (* width height)))";
    let error = plan_introduce_let(request(input, "0.3.3", false)).expect_err("shadowed");

    assert!(
        error
            .to_string()
            .contains("inside an existing binding for 'product'")
    );
}

#[test]
fn rejects_selected_expression_inside_define_compiler_macro_shadowing_lambda_list() {
    let input = "(defun render () (define-compiler-macro slot (&environment product place) (* width height)))";
    let error = plan_introduce_let(request(input, "0.3.3", false)).expect_err("shadowed");

    assert!(
        error
            .to_string()
            .contains("inside an existing binding for 'product'")
    );
}

#[test]
fn rejects_selected_expression_inside_destructuring_bind_shadowing_body() {
    let input = "(defun render () (destructuring-bind (product) row (* width height)))";
    let error = plan_introduce_let(request(input, "0.3.3", false)).expect_err("shadowed");

    assert!(
        error
            .to_string()
            .contains("inside an existing binding for 'product'")
    );
}

#[test]
fn rejects_selected_expression_inside_handler_case_shadowing_clause() {
    let input = "(defun render () (handler-case (risky) (error (product) (* width height))))";
    let error = plan_introduce_let(request(input, "0.3.2.2", false)).expect_err("shadowed");

    assert!(
        error
            .to_string()
            .contains("inside an existing binding for 'product'")
    );
}

#[test]
fn rejects_selected_expression_inside_macrolet_lambda_body_shadowing_parameter() {
    let input = "(defun render () (macrolet ((with-product (product) (* width height))) (done)))";
    let error = plan_introduce_let(request(input, "0.3.1.0.2", false)).expect_err("shadowed");

    assert!(
        error
            .to_string()
            .contains("inside an existing binding for 'product'")
    );
}

#[test]
fn keeps_different_atom_values_out_of_all_occurrences() {
    let input = "(defun render () (+ (* width height) (* width depth)))";
    let plan = plan_introduce_let(request(input, "0.3.1", true)).expect("plan");

    assert_eq!(plan.occurrence_spans.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render () (let ((product (* width height))) (+ product (* width depth))))"
    );
}

fn repeated_products(count: usize) -> String {
    let terms = (0..count)
        .map(|_| "(* width height)")
        .collect::<Vec<_>>()
        .join(" ");
    format!("(defun render () (+ {terms}))")
}

fn repeated_products_with_shadowed_duplicate(count: usize) -> String {
    let terms = (0..count)
        .map(|_| "(* width height)")
        .chain(std::iter::once("(let ((product 0)) (* width height))"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("(defun render () (+ {terms}))")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn all_occurrences_replaces_every_generated_duplicate(count in 1usize..10) {
        let input = repeated_products(count);
        let plan = plan_introduce_let(request(&input, "0.3.1", true)).expect("plan");

        prop_assert_eq!(plan.occurrence_spans.len(), count);
        prop_assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 0);
        prop_assert!(SyntaxTree::parse(&plan.rewritten).is_ok());
        prop_assert_eq!(plan.rewritten.matches("(* width height)").count(), 1);
        prop_assert_eq!(plan.rewritten.matches("product").count(), count + 1);
    }

    #[test]
    fn all_occurrences_skips_generated_shadowed_duplicates(count in 1usize..10) {
        let input = repeated_products_with_shadowed_duplicate(count);
        let plan = plan_introduce_let(request(&input, "0.3.1", true)).expect("plan");

        prop_assert_eq!(plan.occurrence_spans.len(), count);
        prop_assert_eq!(plan.skipped_shadowed_occurrence_spans.len(), 1);
        prop_assert!(SyntaxTree::parse(&plan.rewritten).is_ok());
        prop_assert_eq!(plan.rewritten.matches("(* width height)").count(), 2);
        prop_assert_eq!(plan.rewritten.matches("product").count(), count + 2);
    }
}

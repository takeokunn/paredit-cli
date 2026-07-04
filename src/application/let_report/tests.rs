use proptest::prelude::*;

use super::*;
use crate::domain::sexpr::SyntaxTree;

fn reports_for(input: &str, dialect: Dialect) -> Vec<LetFormReport> {
    let tree = SyntaxTree::parse(input).expect("valid test input");
    build_let_report(dialect, input, &tree).expect("let report builds")
}

#[test]
fn reports_inlineable_single_common_lisp_let() {
    let reports = reports_for("(let ((x 1)) (+ x 2))", Dialect::CommonLisp);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].binding_style, "list-pair");
    assert!(reports[0].inline_supported_by_inline_let);
    assert_eq!(reports[0].bindings[0].reference_count, 1);
    assert!(reports[0].bindings[0].can_inline_without_duplication);
}

#[test]
fn reports_let_star_later_binding_reference() {
    let reports = reports_for("(let* ((x 1) (y (+ x 2))) y)", Dialect::CommonLisp);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].bindings[0].reference_count, 1);
    assert!(reports[0].bindings[0].risks.contains(&"multiple-bindings"));
}

#[test]
fn reports_clojure_vector_bindings() {
    let reports = reports_for("(let [x 1 y (+ x 2)] y)", Dialect::Clojure);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].binding_style, "vector");
    assert_eq!(reports[0].bindings[0].reference_count, 1);
    assert_eq!(reports[0].bindings[1].reference_count, 1);
}

#[test]
fn reports_ignore_shadowed_body_references() {
    let reports = reports_for("(let ((x 1)) (let ((x 2)) x))", Dialect::CommonLisp);

    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].bindings[0].reference_count, 0);
    assert!(reports[0].bindings[0].risks.contains(&"unused-binding"));
    assert_eq!(reports[1].bindings[0].reference_count, 1);
}

#[test]
fn reports_ignore_lambda_parameter_shadowed_references() {
    let reports = reports_for("(let ((x 1)) (lambda (x) x))", Dialect::CommonLisp);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].bindings[0].reference_count, 0);
    assert!(reports[0].bindings[0].risks.contains(&"unused-binding"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn pbt_generated_single_binding_reference_counts_are_reported(ref_count in 0usize..4) {
        let body = match ref_count {
            0 => "42".to_owned(),
            1 => "x".to_owned(),
            n => format!("(+ {})", std::iter::repeat_n("x", n).collect::<Vec<_>>().join(" ")),
        };
        let input = format!("(let ((x 1)) {body})");
        let reports = reports_for(&input, Dialect::CommonLisp);

        prop_assert_eq!(reports.len(), 1);
        prop_assert_eq!(reports[0].bindings[0].reference_count, ref_count);
        prop_assert_eq!(
            reports[0].bindings[0].risks.contains(&"duplicate-evaluation"),
            ref_count > 1
        );
        prop_assert_eq!(
            reports[0].bindings[0].risks.contains(&"unused-binding"),
            ref_count == 0
        );
    }

    #[test]
    fn pbt_shadowed_nested_let_references_do_not_count(value in 0i64..100, shadow_value in 0i64..100) {
        let input = format!("(let ((x {value})) (let ((x {shadow_value})) (+ x x)))");
        let reports = reports_for(&input, Dialect::CommonLisp);

        prop_assert_eq!(reports.len(), 2);
        prop_assert_eq!(reports[0].bindings[0].reference_count, 0);
        prop_assert!(reports[0].bindings[0].risks.contains(&"unused-binding"));
        prop_assert_eq!(reports[1].bindings[0].reference_count, 2);
    }
}

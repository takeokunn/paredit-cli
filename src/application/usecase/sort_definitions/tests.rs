use std::path::PathBuf;

use proptest::{prelude::*, test_runner::TestCaseError};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;

use super::*;

fn request(input: &str, strategy: SortDefinitionsStrategy) -> SortDefinitionsRequest<'_> {
    SortDefinitionsRequest {
        file: PathBuf::from("core.lisp"),
        input,
        dialect: Dialect::CommonLisp,
        strategy,
        write: false,
    }
}

#[test]
fn sorts_contiguous_definitions_by_name() {
    let input = "(in-package #:demo)\n\n\
                 (defun zeta () :z)\n\
                 (defmacro alpha () nil)\n\
                 (defun beta () :b)\n";

    let plan = plan_sort_definitions(request(input, SortDefinitionsStrategy::Name))
        .expect("sort plan should be built");

    assert!(plan.changed);
    assert_eq!(plan.items.len(), 3);
    let alpha = plan.rewritten.find("(defmacro alpha").expect("alpha");
    let beta = plan.rewritten.find("(defun beta").expect("beta");
    let zeta = plan.rewritten.find("(defun zeta").expect("zeta");
    assert!(alpha < beta);
    assert!(beta < zeta);
    assert!(SyntaxTree::parse(&plan.rewritten).is_ok());
}

#[test]
fn does_not_cross_non_definition_barriers() {
    let input = "(defun zeta () :z)\n\
                 (print :barrier)\n\
                 (defun alpha () :a)\n";

    let plan = plan_sort_definitions(request(input, SortDefinitionsStrategy::Name))
        .expect("barrier plan should be built");

    assert!(!plan.changed);
    assert!(plan.items.is_empty());
    assert_eq!(plan.rewritten, input);
}

#[test]
fn kind_then_name_groups_categories_before_names() {
    let input = "(defmacro alpha () nil)\n\
                 (defun zeta () :z)\n\
                 (defun beta () :b)\n\
                 (define-symbol-macro current-user (session-user *session*))\n";

    let plan = plan_sort_definitions(request(input, SortDefinitionsStrategy::KindThenName))
        .expect("sort plan should be built");

    let beta = plan.rewritten.find("(defun beta").expect("beta");
    let zeta = plan.rewritten.find("(defun zeta").expect("zeta");
    let alpha = plan.rewritten.find("(defmacro alpha").expect("alpha");
    let current_user = plan
        .rewritten
        .find("(define-symbol-macro current-user")
        .expect("current-user");
    assert!(beta < zeta);
    assert!(zeta < alpha);
    assert!(alpha < current_user);
}

fn symbol_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,10}".prop_map(|name| name)
}

fn assert_sorted_definitions_property(names: Vec<String>) -> Result<(), TestCaseError> {
    let mut reversed = names.clone();
    reversed.reverse();
    let mut input = String::from("(in-package #:demo)\n");
    for name in &reversed {
        input.push_str(&format!("(defun {name} () :ok)\n"));
    }

    let plan = plan_sort_definitions(request(&input, SortDefinitionsStrategy::Name))
        .map_err(|err| TestCaseError::fail(format!("sort plan: {err}")))?;
    prop_assert!(SyntaxTree::parse(&plan.rewritten).is_ok());

    let mut previous_position = 0;
    for name in names {
        let position = plan
            .rewritten
            .find(&format!("(defun {name}"))
            .ok_or_else(|| TestCaseError::fail(format!("missing {name}")))?;
        prop_assert!(position >= previous_position);
        previous_position = position;
    }

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn sorted_output_is_parseable_and_name_ordered(
        names in prop::collection::btree_set(symbol_name(), 2..8)
            .prop_map(|names| names.into_iter().collect::<Vec<_>>())
    ) {
        assert_sorted_definitions_property(names)?;
    }
}

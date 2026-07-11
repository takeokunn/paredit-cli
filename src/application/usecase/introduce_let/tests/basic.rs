use super::*;

#[test]
fn introduces_single_selected_occurrence_by_default() {
    assert_plan(
        "(defun render () (+ (* width height) margin (* width height)))",
        "0.3.1",
        false,
        1,
        0,
        "(defun render () (let ((product (* width height))) (+ product margin (* width height))))",
    );
}

#[test]
fn introduces_all_structurally_equivalent_occurrences() {
    assert_plan(
        "(defun render () (+ (* width height) margin (*  width height)))",
        "0.3.1",
        true,
        2,
        0,
        "(defun render () (let ((product (* width height))) (+ product margin product)))",
    );
}

#[test]
fn keeps_different_atom_values_out_of_all_occurrences() {
    assert_plan(
        "(defun render () (+ (* width height) (* width depth)))",
        "0.3.1",
        true,
        1,
        0,
        "(defun render () (let ((product (* width height))) (+ product (* width depth))))",
    );
}

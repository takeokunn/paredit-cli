#[test]
fn cli_rename_at_respects_quote_and_quasiquote_depth() {
    let input = "(let ((value xs)) (list value 'value `(value ,value ,@value)))\n";

    let rewritten = write_rename_at(
        "rename-at-reader-boundaries",
        None,
        input,
        "value xs",
        "items",
    );

    assert_eq!(
        rewritten,
        "(let ((items xs)) (list items 'value `(value ,items ,@items)))\n"
    );
}

#[test]
fn cli_rename_at_stops_at_shadowing_bindings() {
    let input = "(let ((value 1)) (+ value (let ((value 2)) value) value))\n";

    let rewritten = write_rename_at("rename-at-shadowing", None, input, "value 1", "outer");

    assert_eq!(
        rewritten,
        "(let ((outer 1)) (+ outer (let ((value 2)) value) outer))\n"
    );
}

#[test]
fn cli_rename_at_common_lisp_keeps_value_and_function_namespaces_separate() {
    let input = "(let ((value 1)) (list value (value)))\n";

    let rewritten = write_rename_at(
        "rename-at-common-lisp-lisp-2",
        None,
        input,
        "value 1",
        "item",
    );

    assert_eq!(rewritten, "(let ((item 1)) (list item (value)))\n");
}

#[test]
fn cli_rename_at_tracks_macrolet_definition_and_calls() {
    let input = "(macrolet ((emit (x) `(list ,x))) (emit 1) #'emit)\n";

    let rewritten = write_rename_at("rename-at-macrolet", None, input, "emit (x)", "produce");

    assert_eq!(
        rewritten,
        "(macrolet ((produce (x) `(list ,x))) (produce 1) #'emit)\n"
    );
}

#[test]
fn cli_rename_at_tracks_symbol_macrolet_value_references() {
    let input =
        "(symbol-macrolet ((place (car cell))) (list place (let ((place 1)) place) 'place))\n";

    let rewritten = write_rename_at(
        "rename-at-symbol-macrolet",
        None,
        input,
        "place (car",
        "slot",
    );

    assert_eq!(
        rewritten,
        "(symbol-macrolet ((slot (car cell))) (list slot (let ((place 1)) place) 'place))\n"
    );
}
use super::*;

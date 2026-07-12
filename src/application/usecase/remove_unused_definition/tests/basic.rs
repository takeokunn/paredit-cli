use super::*;

#[test]
fn plans_private_unused_definition_removal() {
    let text = "(in-package #:app)\n(defun stale-helper () 1)\n(defun live () 2)\n(live)\n";
    let stale_form = "(defun stale-helper () 1)";
    let live_form = "(defun live () 2)";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![
            definition(
                text,
                stale_form,
                "stale-helper",
                DefinitionCategory::Function,
            ),
            definition(text, live_form, "live", DefinitionCategory::Function),
        ],
    ))
    .expect("plan should build");

    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.removal_count, 1);
    assert_eq!(plan.skipped_count, 0);
    assert!(!plan.files[0].rewritten.contains(stale_form));
    SyntaxTree::parse(&plan.files[0].rewritten).expect("rewrite must stay parseable");
}

#[test]
fn skips_a_definition_whose_reported_name_is_a_string_literal_instead_of_erroring() {
    // `(asdf:defsystem "cl-cli" ...)` reports its name as the raw string atom
    // text `"cl-cli"` (quotes included), which is not a valid bare symbol.
    // This must be skipped like any other unsearchable name, not turned into
    // a hard error that aborts the whole command — see
    // `unused-definition-report`, which already skips the same case via
    // `SymbolName::new(name).ok()?`.
    let text = "(asdf:defsystem \"cl-cli\" :depends-on (\"uiop\"))\n";
    let system_form = "(asdf:defsystem \"cl-cli\" :depends-on (\"uiop\"))";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![definition(
            text,
            system_form,
            "\"cl-cli\"",
            DefinitionCategory::System,
        )],
    ))
    .expect("plan should build instead of erroring on a string-literal name");

    assert_eq!(plan.candidate_count, 0);
    assert_eq!(plan.removal_count, 0);
    assert_eq!(plan.files[0].rewritten, text);
}

#[test]
fn does_not_flag_a_function_only_referenced_through_a_function_quote() {
    // A callback/dispatch table capturing a function via `#'name` is a real
    // reference, even though nothing ever calls `(handler ...)` directly.
    let text = "(in-package #:app)\n\
                (defun handler () 1)\n\
                (defparameter *routes* (list (cons :get #'handler)))\n";
    let handler_form = "(defun handler () 1)";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![definition(
            text,
            handler_form,
            "handler",
            DefinitionCategory::Function,
        )],
    ))
    .expect("plan should build");

    // `handler` has a reference (the `#'handler` capture below), so it is not
    // even a removal candidate.
    assert_eq!(plan.candidate_count, 0);
    assert_eq!(plan.removal_count, 0);
    assert!(plan.files[0].rewritten.contains(handler_form));
}

#[test]
fn keeps_functions_referenced_through_callable_accessors() {
    let text = "(in-package #:app)\n\
                (defun handler () 1)\n\
                (defparameter *routes*\n\
                  (list (symbol-function 'handler)\n\
                        (fdefinition 'handler)\n\
                        #'(setf handler)\n\
                        (function (setf handler))\n\
                        (fdefinition '(setf handler))))\n";
    let handler_form = "(defun handler () 1)";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![definition(
            text,
            handler_form,
            "handler",
            DefinitionCategory::Function,
        )],
    ))
    .expect("plan should build");

    assert_eq!(plan.candidate_count, 0);
    assert_eq!(plan.removal_count, 0);
    assert_eq!(plan.skipped_count, 0);
    assert_eq!(plan.files[0].rewritten, text);
}

#[test]
fn keeps_macros_referenced_through_callable_accessors() {
    let text = "(in-package #:app)\n\
                (defmacro helper (&rest body) `(progn ,@body))\n\
                (define-compiler-macro helper (&whole form &rest body) form)\n\
                (defparameter *routes*\n\
                  (list (macro-function 'helper)\n\
                        (compiler-macro-function 'helper)))\n";
    let macro_form = "(defmacro helper (&rest body) `(progn ,@body))";
    let compiler_macro_form = "(define-compiler-macro helper (&whole form &rest body) form)";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![
            definition(text, macro_form, "helper", DefinitionCategory::Macro),
            definition(
                text,
                compiler_macro_form,
                "helper",
                DefinitionCategory::Macro,
            ),
        ],
    ))
    .expect("plan should build");

    assert_eq!(plan.candidate_count, 0);
    assert_eq!(plan.removal_count, 0);
    assert_eq!(plan.skipped_count, 0);
    assert_eq!(plan.files[0].rewritten, text);
}

#[test]
fn ignores_reader_conditionals_that_do_not_overlap_the_removal_spans() {
    let text = "(in-package #:app)\n\
                (defun stale-helper () 1)\n\
                #+sbcl (defun guarded-helper () 2)\n\
                (defun live () 3)\n\
                (live)\n";
    let stale_form = "(defun stale-helper () 1)";
    let guarded_form = "(defun guarded-helper () 2)";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![definition(
            text,
            stale_form,
            "stale-helper",
            DefinitionCategory::Function,
        )],
    ))
    .expect("plan should build");

    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.removal_count, 1);
    assert_eq!(plan.skipped_count, 0);
    assert!(!plan.files[0].rewritten.contains(stale_form));
    assert!(plan.files[0].rewritten.contains(guarded_form));
}

#[test]
fn rejects_removing_a_definition_inside_a_reader_conditional() {
    let text = "#+sbcl (defun stale-helper () 1)\n";
    let stale_form = "(defun stale-helper () 1)";
    let request = request_for(
        text,
        vec![definition(
            text,
            stale_form,
            "stale-helper",
            DefinitionCategory::Function,
        )],
    );

    let error = plan_remove_unused_definitions(request).expect_err("plan must reject");

    assert!(
        error.to_string().contains(
            "cannot safely modify Common Lisp source containing reader conditional #+ at byte 0"
        ),
        "unexpected error: {error:#}"
    );
}

#[test]
fn ignores_vector_literals_that_do_not_overlap_the_removal_span() {
    let text = "(in-package #:app)\n(defun stale-helper () 1)\n(defparameter *values* #(1 2 3))\n(defun live () 2)\n(live)\n";
    let stale_form = "(defun stale-helper () 1)";
    let vector_form = "#(1 2 3)";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![
            definition(
                text,
                stale_form,
                "stale-helper",
                DefinitionCategory::Function,
            ),
            definition(
                text,
                "(defun live () 2)",
                "live",
                DefinitionCategory::Function,
            ),
        ],
    ))
    .expect("plan should build with unrelated vector literals present");

    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.removal_count, 1);
    assert!(!plan.files[0].rewritten.contains(stale_form));
    assert!(plan.files[0].rewritten.contains(vector_form));
    SyntaxTree::parse(&plan.files[0].rewritten).expect("rewrite must stay parseable");
}

#[test]
fn keeps_definitions_referenced_from_another_input_file() {
    let definition_text = "(in-package #:app)\n(defun shared-helper () 1)\n";
    let consumer_text = "(in-package #:app)\n(defun call-shared () (shared-helper))\n";
    let shared_form = "(defun shared-helper () 1)";

    let plan = plan_remove_unused_definitions(RemoveUnusedDefinitionsRequest {
        files: vec![
            file_with_text(
                PathBuf::from("library.lisp"),
                definition_text,
                vec![definition(
                    definition_text,
                    shared_form,
                    "shared-helper",
                    DefinitionCategory::Function,
                )],
            ),
            file_with_text(PathBuf::from("consumer.lisp"), consumer_text, Vec::new()),
        ],
        package_definitions: Vec::new(),
        include_protected: false,
        include_exported: false,
    })
    .expect("plan should build");

    assert_eq!(plan.candidate_count, 0);
    assert_eq!(plan.removal_count, 0);
    assert_eq!(plan.skipped_count, 0);
    assert!(plan.files[0].rewritten.contains(shared_form));
    assert_eq!(plan.files[1].rewritten, consumer_text);
}

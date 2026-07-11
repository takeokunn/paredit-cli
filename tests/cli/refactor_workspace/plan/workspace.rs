use super::*;

#[test]
fn cli_builds_workspace_refactor_plan_for_symbol_macro_targets() {
    assert_workspace_plan_target(
        "workspace-symbol-macro-plan",
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-symbol-macro current-user "guest")
(defun caller () current-user)
"#,
        "current-user",
        "rename",
        "symbol_macro",
        "apply-symbol-macro-rename",
        "paredit rename-symbol-macro --from 'current-user' --to <new-symbol> --output json",
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_for_symbol_macro_move_targets() {
    assert_workspace_plan_target(
        "workspace-symbol-macro-move-plan",
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-symbol-macro current-user "guest")
(defun caller () current-user)
"#,
        "current-user",
        "move",
        "symbol_macro",
        "apply-move",
        "paredit move-definition --from-file <file> --to-file <file> --path <definition-path> --plan --output json",
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_for_macro_targets() {
    assert_workspace_plan_target(
        "workspace-macro-plan",
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defmacro render-pane (pane) `(draw-pane ,pane))
(defun caller () (render-pane window))
"#,
        "render-pane",
        "rename",
        "macro",
        "apply-macro-rename",
        "paredit rename-function --from 'render-pane' --to <new-symbol> --output json",
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_for_define_method_combination_targets() {
    assert_workspace_plan_target(
        "workspace-define-method-combination-plan",
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-method-combination render-combination (pane theme) ((primary *)) (list pane theme primary))
(defun caller () (render-combination pane theme))
"#,
        "render-combination",
        "rename",
        "macro",
        "apply-macro-rename",
        "paredit rename-function --from 'render-combination' --to <new-symbol> --output json",
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_for_define_modify_macro_targets() {
    assert_workspace_plan_target(
        "workspace-modify-macro-plan",
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-modify-macro updatef (place) incf)
(defun caller () (updatef counter))
"#,
        "updatef",
        "rename",
        "macro",
        "apply-macro-rename",
        "paredit rename-function --from 'updatef' --to <new-symbol> --output json",
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_for_compiler_macro_targets() {
    assert_workspace_plan_target(
        "workspace-compiler-macro-plan",
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-compiler-macro fast-add (x y) `(+ ,x ,y))
(defun caller () (fast-add 1 2))
"#,
        "fast-add",
        "rename",
        "compiler_macro",
        "apply-macro-rename",
        "paredit rename-function --from 'fast-add' --to <new-symbol> --output json",
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_from_directory_roots() {
    let dir = fresh_temp_dir("workspace refactor plan");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let file = src_dir.join("core.lisp");
    let ignored = src_dir.join("notes.txt");
    write_fixture(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    );
    fs::write(&ignored, "render-pane is mentioned in plain text")
        .expect("write ignored workspace fixture");

    let output = run_workspace_refactor_plan_json(&dir, "render-pane", "rename", &[]);

    assert!(output.stdout.contains("\"workspace\""));
    assert!(output.stdout.contains("\"discovered_file_count\": 1"));
    assert!(output.stdout.contains("\"unknown\": 1"));
    assert!(output.stdout.contains("\"file_count\": 1"));
    assert!(output.stdout.contains("\"definition_count\": 1"));
    assert!(output.stdout.contains("\"call_count\": 1"));
    assert!(
        output.stdout.contains(
            "paredit rename-symbols --from 'render-pane' --to <new-symbol> --output json"
        )
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_with_hidden_and_generated_inputs() {
    let dir = fresh_temp_dir("workspace refactor plan-discovery-flags");
    let src_dir = dir.join("src");
    let hidden_dir = dir.join(".hidden");
    let generated_dir = dir.join("target");
    fs::create_dir_all(&src_dir).expect("create source dir");
    fs::create_dir_all(&hidden_dir).expect("create hidden dir");
    fs::create_dir_all(&generated_dir).expect("create generated dir");

    write_fixture(
        &src_dir.join("core.lisp"),
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window))
"#,
    );
    write_fixture(
        &hidden_dir.join("secret.lisp"),
        r#"(defpackage #:demo.hidden (:use #:cl))
(in-package #:demo.hidden)
(defun render-pane (pane) (draw-pane pane))
(defun hidden-caller () (render-pane hidden-window))
"#,
    );
    write_fixture(
        &generated_dir.join("generated.lisp"),
        r#"(defpackage #:demo.generated (:use #:cl))
(in-package #:demo.generated)
(defun render-pane (pane) (draw-pane pane))
(defun generated-caller () (render-pane generated-window))
"#,
    );

    let output = run_workspace_refactor_plan_json(
        &dir,
        "render-pane",
        "rename",
        &["--include-hidden", "--include-generated"],
    );

    assert!(output.stdout.contains("\"workspace\""));
    assert!(output.stdout.contains("\"discovered_file_count\": 3"));
    assert!(output.stdout.contains("\"hidden\": 0"));
    assert!(output.stdout.contains("\"generated\": 0"));
    assert!(output.stdout.contains("\"unknown\": 0"));
    assert!(output.stdout.contains("\"file_count\": 3"));
    assert!(output.stdout.contains("\"definition_count\": 3"));
    assert!(output.stdout.contains("\"call_count\": 3"));
    assert!(output.stdout.contains("\"status\": \"manual_review\""));
    assert!(output.stdout.contains("\"safe_to_automate\": false"));
    assert!(output.stdout.contains("\"blocking_gate_count\": 2"));
    assert!(output.stdout.contains("\"code\": \"ambiguous-definition\""));
    assert!(
        output
            .stdout
            .contains("\"action\": \"review-rename-scope\"")
    );
    assert!(
        output
            .stdout
            .contains("paredit impact-report --symbol 'render-pane'")
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_with_max_depth_limit() {
    let dir = fresh_temp_dir("workspace refactor plan-max-depth");
    let nested_dir = dir.join("src");
    fs::create_dir_all(&nested_dir).expect("create nested source dir");

    write_fixture(
        &dir.join("root.lisp"),
        r#"(defpackage #:demo.root (:use #:cl))
(in-package #:demo.root)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    );
    write_fixture(
        &nested_dir.join("core.lisp"),
        r#"(defpackage #:demo.nested (:use #:cl))
(in-package #:demo.nested)
(defun render-pane (pane) (draw-pane pane))
(defun nested-caller () (render-pane nested-window))
"#,
    );

    let output =
        run_workspace_refactor_plan_json(&dir, "render-pane", "rename", &["--max-depth", "1"]);

    assert!(output.stdout.contains("\"workspace\""));
    assert!(output.stdout.contains("\"discovered_file_count\": 1"));
    assert!(output.stdout.contains("\"file_count\": 1"));
    assert!(output.stdout.contains("\"definition_count\": 1"));
    assert!(output.stdout.contains("\"call_count\": 1"));
    assert!(output.stdout.contains("\"status\": \"ready\""));
    assert!(output.stdout.contains("\"safe_to_automate\": true"));
    assert!(output.stdout.contains("\"blocking_gate_count\": 0"));
    assert!(
        output.stdout.contains(
            "paredit rename-symbols --from 'render-pane' --to <new-symbol> --output json"
        )
    );
}

#[test]
fn cli_builds_workspace_refactor_plan_with_unknown_inputs() {
    let dir = fresh_temp_dir("workspace refactor plan-unknown");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    write_fixture(
        &src_dir.join("core.lisp"),
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window))
"#,
    );
    write_fixture(
        &src_dir.join("scratch.txt"),
        r#"(defun unknown-caller () (render-pane preview-window))
"#,
    );

    let output =
        run_workspace_refactor_plan_json(&dir, "render-pane", "rename", &["--include-unknown"]);

    assert!(output.stdout.contains("\"workspace\""));
    assert!(output.stdout.contains("\"discovered_file_count\": 2"));
    assert!(output.stdout.contains("\"unknown\": 0"));
    assert!(output.stdout.contains("\"file_count\": 2"));
    assert!(output.stdout.contains("\"definition_count\": 1"));
    assert!(output.stdout.contains("\"call_count\": 2"));
    assert!(output.stdout.contains("\"status\": \"ready\""));
    assert!(output.stdout.contains("\"safe_to_automate\": true"));
    assert!(output.stdout.contains("\"blocking_gate_count\": 0"));
    assert!(output.stdout.contains("core.lisp"));
    assert!(output.stdout.contains("scratch.txt"));
    assert!(
        output.stdout.contains(
            "paredit rename-function --from 'render-pane' --to <new-symbol> --output json"
        )
    );
}

#[cfg(unix)]
#[test]
fn cli_reports_skipped_symlink_in_workspace_refactor_plan() {
    let dir = fresh_temp_dir("workspace refactor plan-symlink");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let file = src_dir.join("core.lisp");
    write_fixture(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    );
    std::os::unix::fs::symlink(&file, dir.join("linked-core.lisp"))
        .expect("create workspace symlink");

    let output = run_workspace_refactor_plan_json(&dir, "render-pane", "rename", &[]);

    assert!(output.stdout.contains("\"workspace\""));
    assert!(output.stdout.contains("\"discovered_file_count\": 1"));
    assert!(output.stdout.contains("\"symlink\": 1"));
    assert!(output.stdout.contains("\"file_count\": 1"));
    assert!(output.stdout.contains("\"definition_count\": 1"));
    assert!(output.stdout.contains("\"status\": \"ready\""));
}

#[test]
fn cli_builds_workspace_remove_plan_with_unused_definition_cleanup_command() {
    let dir = fresh_temp_dir("workspace-remove-plan");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    write_fixture(
        &src_dir.join("core.lisp"),
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun stale-helper (value) value)
(defun caller () 42)
"#,
    );

    let output = run_workspace_refactor_plan_json(&dir, "stale-helper", "remove", &[]);

    assert!(output.stdout.contains("\"operation\": \"remove\""));
    assert!(output.stdout.contains("\"symbol\": \"stale-helper\""));
    assert!(
        output
            .stdout
            .contains("\"action\": \"apply-unused-definition-removal\"")
    );
    assert!(
        output
            .stdout
            .contains("paredit remove-unused-definitions --output json")
    );
    assert!(output.stdout.contains(
        "paredit refactor verify --symbol 'stale-helper' --operation remove --phase post --output json",
    ));
}

use super::*;

#[test]
fn cli_skips_common_lisp_macrolet_shadowed_global_definition_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-macrolet-shadowing",
        "macrolet.lisp",
        "(defun helper (x) (+ x 1))\n(defun render () (macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))\n",
    );

    let output = paredit()
        .arg("inspect")
        .arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert_shadowed_helper_edges(&stdout);
}

#[test]
fn cli_skips_common_lisp_cl_user_macrolet_shadowed_global_definition_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-cl-user-macrolet-shadowing",
        "cl-user-macrolet.lisp",
        "(defun helper (x) (+ x 1))\n(defun render () (cl-user:macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))\n",
    );

    let output = paredit()
        .arg("inspect")
        .arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert_shadowed_helper_edges(&stdout);
}

#[test]
fn cli_skips_common_lisp_cl_macrolet_shadowed_global_definition_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-cl-macrolet-shadowing",
        "cl-macrolet.lisp",
        "(defun helper (x) (+ x 1))\n(defun render () (cl:macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))\n",
    );

    let output = paredit()
        .arg("inspect")
        .arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert_shadowed_helper_edges(&stdout);
}

#[test]
fn cli_skips_emacs_lisp_cl_macrolet_shadowed_global_definition_edges() {
    let elisp_file = write_call_graph_fixture(
        "call-graph-cl-macrolet-shadowing",
        "macrolet.el",
        "(defun helper (x) (+ x 1))\n(defun render () (cl-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))\n",
    );

    let output = paredit()
        .arg("inspect")
        .arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&elisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert!(stdout.contains("\"dialect\": \"emacs-lisp\""));
    assert_shadowed_helper_edges(&stdout);
}

#[test]
fn cli_skips_common_lisp_compiler_macrolet_shadowed_global_definition_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-compiler-macrolet-shadowing",
        "compiler-macrolet.lisp",
        "(defun helper (x) (+ x 1))\n(defun render () (compiler-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))\n",
    );

    let output = paredit()
        .arg("inspect")
        .arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert_shadowed_helper_edges(&stdout);
}

#[test]
fn cli_skips_common_lisp_cl_compiler_macrolet_shadowed_global_definition_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-cl-compiler-macrolet-shadowing",
        "cl-compiler-macrolet.lisp",
        "(defun helper (x) (+ x 1))\n(defun render () (cl:compiler-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))\n",
    );

    let output = paredit()
        .arg("inspect")
        .arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert_shadowed_helper_edges(&stdout);
}

#[test]
fn cli_skips_common_lisp_cl_user_compiler_macrolet_shadowed_global_definition_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-cl-user-compiler-macrolet-shadowing",
        "cl-user-compiler-macrolet.lisp",
        "(defun helper (x) (+ x 1))\n(defun render () (cl-user:compiler-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))\n",
    );

    let output = paredit()
        .arg("inspect")
        .arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert_shadowed_helper_edges(&stdout);
}

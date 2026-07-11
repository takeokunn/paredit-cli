use super::*;

#[test]
fn cli_reports_call_graph_across_dialects() {
    let dir = fresh_temp_dir("call-graph");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n(defun render () (area 10 20) (external 1))\n",
    )
    .expect("write lisp fixture");
    fs::write(&elisp_file, "(defun draw () (area 5 6))\n").expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("call-graph")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"includeExternal\": false"))
        .stdout(predicate::str::contains("\"edge_count\": 2"))
        .stdout(predicate::str::contains("\"internal_edge_count\": 2"))
        .stdout(predicate::str::contains("\"external_edge_count\": 0"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"caller\": \"render\""))
        .stdout(predicate::str::contains("\"caller\": \"draw\""))
        .stdout(predicate::str::contains("\"callee\": \"area\""))
        .stdout(predicate::str::contains("\"calleeCategories\""))
        .stdout(predicate::str::contains("\"function\""));

    let mut filtered = paredit();
    filtered
        .arg("call-graph")
        .arg("--symbol")
        .arg("render")
        .arg("--include-external")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"symbol\": \"render\""))
        .stdout(predicate::str::contains("\"includeExternal\": true"))
        .stdout(predicate::str::contains("\"caller\": \"render\""))
        .stdout(predicate::str::contains("\"callee\": \"external\""));
}

#[test]
fn cli_skips_common_lisp_defmethod_specialized_lambda_list_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-defmethod-specializer",
        "methods.lisp",
        "(defmethod render :around ((node widget) stream) (draw node stream))\n(defun draw (node stream) stream)\n",
    );

    let mut cmd = paredit();
    cmd.arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"caller\": \"render\""))
        .stdout(predicate::str::contains("\"callee\": \"draw\""))
        .stdout(predicate::str::contains("\"callee\": \"node\"").not());
}

#[test]
fn cli_reports_common_lisp_symbol_macrolet_expansion_and_body_edges_without_binding_name_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-symbol-macrolet",
        "symbol-macrolet.lisp",
        "(defun helper (x) (+ x 10))\n(defun target (x) x)\n(defun render () (symbol-macrolet ((helper (target 1))) (list helper (target 2))))\n",
    );

    let mut cmd = paredit();
    cmd.arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"edge_count\": 2"))
        .stdout(predicate::str::contains("\"caller\": \"render\""))
        .stdout(predicate::str::contains("\"callee\": \"target\""))
        .stdout(predicate::str::contains("\"callee\": \"helper\"").not());
}

#[test]
fn cli_reports_common_lisp_setf_callable_edges() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-common-lisp-setf",
        "setf.lisp",
        "(define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n(defun render (item) (setf (accessor item) 1) accessor)\n(defun wrapper (item) (setf (accessor item) 2))\n",
    );

    let mut cmd = paredit();
    cmd.arg("call-graph")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"edge_count\": 2"))
        .stdout(predicate::str::contains("\"caller\": \"render\""))
        .stdout(predicate::str::contains("\"caller\": \"wrapper\""))
        .stdout(predicate::str::contains("\"callee\": \"accessor\""))
        .stdout(predicate::str::contains("\"argumentCount\": 1"))
        .stdout(predicate::str::contains("\"macro\""));
}

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
fn cli_gates_call_graph_policy_for_ci() {
    let dir = fresh_temp_dir("call-graph-policy");
    let lisp_file = dir.join("core.lisp");
    fs::write(&lisp_file, "(defun leaf () 1)\n(defun render () (leaf))\n")
        .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("call-graph")
        .arg("--symbol")
        .arg("leaf")
        .arg("--fail-on-inbound-callers")
        .arg("--require-edges")
        .arg("2")
        .arg("--require-internal-edges")
        .arg("2")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"symbol\": \"leaf\""))
        .stdout(predicate::str::contains("\"edge_count\": 1"))
        .stdout(predicate::str::contains("\"internal_edge_count\": 1"))
        .stdout(predicate::str::contains("\"inbound_edge_count\": 1"))
        .stdout(predicate::str::contains(
            "\"fail_on_inbound_callers\": true",
        ))
        .stdout(predicate::str::contains("\"require_edges\": 2"))
        .stdout(predicate::str::contains("\"require_internal_edges\": 2"))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "focused symbol has inbound callers: render",
        ))
        .stdout(predicate::str::contains("edge count 1 is below required 2"))
        .stdout(predicate::str::contains(
            "internal edge count 1 is below required 2",
        ))
        .stderr(predicate::str::contains("call-graph policy failed"));
}

#[test]
fn cli_accepts_call_graph_policy_without_inbound_callers() {
    let dir = fresh_temp_dir("call-graph-policy-compatible");
    let lisp_file = dir.join("core.lisp");
    fs::write(&lisp_file, "(defun leaf () 1)\n(defun render () (leaf))\n")
        .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("call-graph")
        .arg("--symbol")
        .arg("render")
        .arg("--fail-on-inbound-callers")
        .arg("--require-edges")
        .arg("1")
        .arg("--require-internal-edges")
        .arg("1")
        .arg("--output")
        .arg("text")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("edge_count\t1"))
        .stdout(predicate::str::contains("internal_edge_count\t1"))
        .stdout(predicate::str::contains("inbound_edge_count\t0"))
        .stdout(predicate::str::contains("policy_passed\ttrue"));
}

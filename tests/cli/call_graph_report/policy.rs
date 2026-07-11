use super::*;

#[test]
fn cli_gates_call_graph_policy_for_ci() {
    let lisp_file = write_call_graph_fixture(
        "call-graph-policy",
        "core.lisp",
        "(defun leaf () 1)\n(defun render () (leaf))\n",
    );

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("call-graph")
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
    let lisp_file = write_call_graph_fixture(
        "call-graph-policy-compatible",
        "core.lisp",
        "(defun leaf () 1)\n(defun render () (leaf))\n",
    );

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("call-graph")
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

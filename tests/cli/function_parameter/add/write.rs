use super::*;

#[test]
fn cli_writes_add_function_parameter_for_scheme_start() {
    let dir = fresh_temp_dir("add-function-parameter");
    let scheme_file = dir.join("render.scm");
    fs::write(
        &scheme_file,
        "(define (area width height) (* width height))\n(define rendered (area 10 20))\n",
    )
    .expect("write scheme fixture");

    let output = paredit()
        .arg("refactor")
        .arg("add-function-parameter")
        .arg("--file")
        .arg(&scheme_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--name")
        .arg("scale")
        .arg("--argument")
        .arg("2")
        .arg("--call-path")
        .arg("1.2")
        .arg("--insert")
        .arg("start")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run add-function-parameter");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_add_function_parameter_report(&output.stdout).expect("parse add report");
    assert_eq!(report.function_name, "area");
    assert_eq!(report.insert, "start");
    assert!(report.written);

    let rewritten = fs::read_to_string(&scheme_file).expect("read parameterized scheme");
    assert_eq!(
        rewritten,
        "(define (area scale width height) (* width height))\n(define rendered (area 2 10 20))\n"
    );
    assert_eq!(rewritten, report.rewritten);
}

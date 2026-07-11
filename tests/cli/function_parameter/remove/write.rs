pub(super) use super::*;

#[test]
fn cli_writes_remove_function_parameter_for_scheme() {
    let dir = fresh_temp_dir("remove-function-parameter");
    let scheme_file = dir.join("render.scm");
    fs::write(
        &scheme_file,
        "(define (area scale width height) (* width height))\n(define rendered (area 2 10 20))\n",
    )
    .expect("write scheme fixture");

    let output = remove_command()
        .arg("remove-function-parameter")
        .arg("--file")
        .arg(&scheme_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--name")
        .arg("scale")
        .arg("--call-path")
        .arg("1.2")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run remove-function-parameter");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report =
        parse_remove_function_parameter_report(&output.stdout).expect("parse remove report");
    assert_eq!(report.function_name, "area");
    assert_eq!(report.parameter_name, "scale");
    assert!(report.written);

    let rewritten = fs::read_to_string(&scheme_file).expect("read parameter-pruned scheme");
    assert_eq!(
        rewritten,
        "(define (area width height) (* width height))\n(define rendered (area 10 20))\n"
    );
    assert_eq!(rewritten, report.rewritten);
}

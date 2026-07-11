use super::*;

#[test]
fn cli_writes_swap_function_parameters_with_all_calls() {
    let dir = fresh_temp_dir("swap-function-parameters-all-calls");
    let common_lisp_file = dir.join("render.lisp");
    fs::write(
        &common_lisp_file,
        "(defun area (scale width height) (* scale width height))\n(defun a () (area 2 10 20))\n(defun b () (list (area 3 4 5)))\n",
    )
    .expect("write common lisp fixture");

    let output = swap_command()
        .arg("--file")
        .arg(&common_lisp_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--left-name")
        .arg("scale")
        .arg("--right-name")
        .arg("height")
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run swap-function-parameters");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_swap_function_parameters_report(&output.stdout).expect("parse swap report");
    assert!(report.all_calls);
    assert!(report.changed);
    assert!(report.written);
    assert_eq!(report.function_name, "area");
    assert_eq!(report.left_name, "scale");
    assert_eq!(report.right_name, "height");
    assert_eq!(report.left_index, 0);
    assert_eq!(report.right_index, 2);
    assert_eq!(report.swapped_arguments.len(), 2);

    assert_eq!(
        fs::read_to_string(common_lisp_file).expect("read parameter-swapped common lisp"),
        "(defun area (height width scale) (* scale width height))\n(defun a () (area 20 10 2))\n(defun b () (list (area 5 4 3)))\n"
    );
}

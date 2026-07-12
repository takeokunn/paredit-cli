use super::*;

#[test]
fn rejects_explicit_labels_shadowed_call_path() {
    let input = "\
(defun f (a) a)
(defun caller ()
  (labels ((f (x) (f x))
           (g (y) (f y)))
    (f 1)
    (cl:print (f 2)))
  (f 3))";
    let error = plan_add_function_parameter(request(
        input,
        "b",
        "0",
        vec![path("1.3.1.0.2")],
        false,
        FunctionParameterInsert::End,
        FunctionParameterSection::Auto,
    ))
    .expect_err("labels local call should be rejected");

    assert!(
        error
            .to_string()
            .contains("shadowed by a local callable binding or overlaps the selected definition")
    );
}

#[test]
fn rejects_explicit_cl_user_labels_shadowed_call_path() {
    let input = "\
(defun f (a) a)
(defun caller ()
  (cl-user:labels ((f (x) (f x))
                   (g (y) (f y)))
    (f 1)
    (cl:print (f 2)))
  (f 3))";
    let error = plan_add_function_parameter(request(
        input,
        "b",
        "0",
        vec![path("1.3.1.0.2")],
        false,
        FunctionParameterInsert::End,
        FunctionParameterSection::Auto,
    ))
    .expect_err("labels local call should be rejected");

    assert!(
        error
            .to_string()
            .contains("shadowed by a local callable binding or overlaps the selected definition")
    );
}

#[test]
fn rejects_explicit_macrolet_shadowed_call_path() {
    let input = "\
(defun f (a) a)
(defun caller ()
  (macrolet ((f (x) (f x)))
    (f 1))
  (f 2))";
    let error = plan_add_function_parameter(request(
        input,
        "b",
        "0",
        vec![path("1.3.2")],
        false,
        FunctionParameterInsert::End,
        FunctionParameterSection::Auto,
    ))
    .expect_err("macrolet body local call should be rejected");

    assert!(
        error
            .to_string()
            .contains("shadowed by a local callable binding or overlaps the selected definition")
    );
}

#[test]
fn rejects_explicit_cl_user_macrolet_shadowed_call_path() {
    let input = "\
(defun f (a) a)
(defun caller ()
  (cl-user:macrolet ((f (x) (f x)))
    (f 1))
  (f 2))";
    let error = plan_add_function_parameter(request(
        input,
        "b",
        "0",
        vec![path("1.3.2")],
        false,
        FunctionParameterInsert::End,
        FunctionParameterSection::Auto,
    ))
    .expect_err("macrolet body local call should be rejected");

    assert!(
        error
            .to_string()
            .contains("shadowed by a local callable binding or overlaps the selected definition")
    );
}

#[test]
fn rejects_explicit_compiler_macrolet_shadowed_call_path() {
    let input = "\
(defun f (a) a)
(defun caller ()
  (compiler-macrolet ((f (x) (f x)))
    (f 1))
  (f 2))";
    let error = plan_add_function_parameter(request(
        input,
        "b",
        "0",
        vec![path("1.3.2")],
        false,
        FunctionParameterInsert::End,
        FunctionParameterSection::Auto,
    ))
    .expect_err("compiler-macrolet body local call should be rejected");

    assert!(
        error
            .to_string()
            .contains("shadowed by a local callable binding or overlaps the selected definition")
    );
}

#[test]
fn rejects_explicit_cl_user_compiler_macrolet_shadowed_call_path() {
    let input = "\
(defun f (a) a)
(defun caller ()
  (cl-user:compiler-macrolet ((f (x) (f x)))
    (f 1))
  (f 2))";
    let error = plan_add_function_parameter(request(
        input,
        "b",
        "0",
        vec![path("1.3.2")],
        false,
        FunctionParameterInsert::End,
        FunctionParameterSection::Auto,
    ))
    .expect_err("qualified compiler-macrolet body local call should be rejected");

    assert!(
        error
            .to_string()
            .contains("shadowed by a local callable binding or overlaps the selected definition")
    );
}

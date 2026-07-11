use super::*;

#[test]
fn all_calls_skip_cl_user_macrolet_local_function_calls() {
    assert_wrap_calls!(
        input: "(defun render () (cl-user:macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)) (fetch-user root))",
        scope: WrapFunctionCallsScope::AllCalls,
        calls: 2,
        rewritten_contains: [
            "(cl-user:macrolet",
            "(with-cache (fetch-user root))"
        ]
    );
}

#[test]
fn explicit_path_rejects_compiler_macrolet_shadowed_local_function_calls() {
    assert_shadowed_wrap_explicit_path!(
        "(defun render () (compiler-macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))"
    );
}

#[test]
fn explicit_path_rejects_cl_user_compiler_macrolet_shadowed_local_function_calls() {
    assert_shadowed_wrap_explicit_path!(
        "(defun render () (cl-user:compiler-macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))"
    );
}

#[test]
fn explicit_path_rejects_cl_user_macrolet_shadowed_local_function_calls() {
    assert_shadowed_wrap_explicit_path!(
        "(defun render () (cl-user:macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))"
    );
}

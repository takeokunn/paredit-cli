use super::*;

#[test]
fn all_calls_skip_cl_user_macrolet_local_function_calls() {
    assert_unwrap_calls!(
        input: "(defun render () (cl-user:macrolet ((fetch-user (id) `(trace (fetch-user ,id)))) (trace (fetch-user user))) (trace (fetch-user root)))",
        scope: UnwrapFunctionCallsScope::AllCalls,
        calls: 2,
        rewritten_contains: [
            "(cl-user:macrolet",
            "(fetch-user root)"
        ]
    );
}

#[test]
fn explicit_path_rejects_cl_user_macrolet_shadowed_local_function_calls() {
    assert_shadowed_unwrap_explicit_path!(
        "(defun render () (cl-user:macrolet ((fetch-user (id) `(trace (fetch-user ,id)))) (trace (fetch-user user))))"
    );
}

#[test]
fn explicit_path_rejects_compiler_macrolet_shadowed_local_function_calls() {
    assert_shadowed_unwrap_explicit_path!(
        "(defun render () (compiler-macrolet ((fetch-user (id) `(trace (fetch-user ,id)))) (trace (fetch-user user))))"
    );
}

#[test]
fn explicit_path_rejects_cl_user_compiler_macrolet_shadowed_local_function_calls() {
    assert_shadowed_unwrap_explicit_path!(
        "(defun render () (cl-user:compiler-macrolet ((fetch-user (id) `(trace (fetch-user ,id)))) (trace (fetch-user user))))"
    );
}

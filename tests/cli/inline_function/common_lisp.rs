use super::*;

#[test]
fn cli_plans_inline_function_with_common_lisp_key_parameter() {
    assert_inline_success(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        "(defun render-one (x &key (style :plain)) (list x style))\n\
         (defun render () (render-one 1 :style :bold))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 :bold)\"",
            "\"name\": \"style\"",
            "\"argument\": \":bold\"",
            "(defun render () (list 1 :bold))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_allow_other_keys_and_rest() {
    assert_common_lisp_inline_success(
        "(defun render-one (x &rest rest &key (style :plain) &allow-other-keys) (list x style rest))\n\
         (print (render-one 1 :style :bold :size 10))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 :bold (:style :bold :size 10))\"",
            "\"name\": \"rest\"",
            "\"argument\": \"(:style :bold :size 10)\"",
            "(print (list 1 :bold (:style :bold :size 10)))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_allow_other_keys_without_rest_when_dropping_is_allowed()
 {
    assert_common_lisp_inline_success_with_args(
        &["--allow-drop-arguments"],
        "(defun render-one (x &key (style :plain) &allow-other-keys) (list x style))\n\
         (print (render-one 1 :style :bold :size 10))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 :bold)\"",
            "\"name\": \"style\"",
            "\"argument\": \":bold\"",
            "(print (list 1 :bold))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_optional_supplied_p_parameter() {
    assert_common_lisp_inline_success(
        "(defun maybe (x &optional (y 10 y-p)) (if y-p y x))\n\
         (print (maybe 1 2))",
        &[
            "\"function_name\": \"maybe\"",
            "\"replacement\": \"(if t 2 1)\"",
            "\"name\": \"y-p\"",
            "\"argument\": \"t\"",
            "(print (if t 2 1))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_docstring_and_declare() {
    assert_common_lisp_inline_success(
        "(defmacro wrap (head &body body)\n\
           \"Wrap BODY with HEAD.\"\n\
           (declare (ignorable head))\n\
           `(list ,head ,@body))\n\
         (print (wrap :x 1 2 3))",
        &[
            "\"function_name\": \"wrap\"",
            "\"replacement\": \"(list :x 1 2 3)\"",
            "(print (list :x 1 2 3))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_whole() {
    assert_common_lisp_inline_success(
        "(defmacro inspect (&whole form x) `(list (quote ,form) ,x))\n\
         (print (inspect 42))",
        &[
            "\"function_name\": \"inspect\"",
            "\"replacement\": \"(list (quote (inspect 42)) 42)\"",
            "(print (list (quote (inspect 42)) 42))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_whole_and_required_destructuring() {
    assert_common_lisp_inline_success(
        "(defmacro inspect (&whole form (left right)) `(list (quote ,form) ,right ,left))\n\
         (print (inspect (a b)))",
        &[
            "\"function_name\": \"inspect\"",
            "\"replacement\": \"(list (quote (inspect (a b))) b a)\"",
            "\"name\": \"left\"",
            "\"argument\": \"a\"",
            "\"name\": \"right\"",
            "\"argument\": \"b\"",
            "(print (list (quote (inspect (a b))) b a))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_whole_and_aux_parameter() {
    assert_common_lisp_inline_success(
        "(defmacro inspect (&whole form (value &aux (tag :seen))) `(list (quote ,form) ,value ,tag))\n\
         (print (inspect (a)))",
        &[
            "\"function_name\": \"inspect\"",
            "\"replacement\": \"(list (quote (inspect (a))) a :seen)\"",
            "\"name\": \"value\"",
            "\"argument\": \"a\"",
            "\"name\": \"tag\"",
            "\"argument\": \":seen\"",
            "(print (list (quote (inspect (a))) a :seen))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_aux_binding_chain() {
    assert_common_lisp_inline_success_with_args(
        &["--allow-duplicate-evaluation"],
        "(defmacro render-one (x &aux (y x) (z y)) `(list ,y ,z))\n\
         (print (render-one 1))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 1)\"",
            "\"name\": \"x\"",
            "\"argument\": \"1\"",
            "(print (list 1 1))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_define_compiler_macro() {
    assert_common_lisp_inline_success(
        "(define-compiler-macro area (w h) `(* ,w ,h))\n\
         (print (area 3 4))",
        &[
            "\"function_name\": \"area\"",
            "\"replacement\": \"(* 3 4)\"",
            "(define-compiler-macro area (w h) `(* ,w ,h))",
            "(print (* 3 4))",
        ],
        &[],
    );
}

#[test]
fn cli_rejects_inline_function_for_common_lisp_define_setf_expander() {
    assert_common_lisp_inline_failure(
        "(define-setf-expander slot (place) (values nil nil '(setf slot) (list place)))\n\
         (print (slot foo))",
        &[
            "define-setf-expander",
            "setf expanders rewrite places, not ordinary call expressions",
        ],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_define_compiler_macro_whole_and_required_destructuring()
 {
    assert_common_lisp_inline_success(
        "(define-compiler-macro inspect (&whole form (left right)) `(list (quote ,form) ,right ,left))\n\
         (print (inspect (a b)))",
        &[
            "\"function_name\": \"inspect\"",
            "\"replacement\": \"(list (quote (inspect (a b))) b a)\"",
            "\"name\": \"left\"",
            "\"argument\": \"a\"",
            "\"name\": \"right\"",
            "\"argument\": \"b\"",
            "(print (list (quote (inspect (a b))) b a))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_define_compiler_macro_top_level_key_destructuring() {
    assert_common_lisp_inline_success(
        "(define-compiler-macro wrap (&key ((:style (mode variant)) '(:plain :narrow) style-p)) `(list ,mode ,variant ,style-p))\n\
         (print (wrap :style (:bold :wide)))",
        &[
            "\"function_name\": \"wrap\"",
            "\"replacement\": \"(list :bold :wide t)\"",
            "\"name\": \"mode\"",
            "\"argument\": \":bold\"",
            "\"name\": \"variant\"",
            "\"argument\": \":wide\"",
            "(print (list :bold :wide t))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_define_compiler_macro_whole_and_top_level_key_destructuring()
 {
    assert_common_lisp_inline_success(
        "(define-compiler-macro wrap (&whole form &key ((:style (mode variant)) '(:plain :narrow) style-p)) `(list (quote ,form) ,mode ,variant ,style-p))\n\
         (print (wrap :style (:bold :wide)))",
        &[
            "\"function_name\": \"wrap\"",
            "\"replacement\": \"(list (quote (wrap :style (:bold :wide))) :bold :wide t)\"",
            "\"name\": \"mode\"",
            "\"argument\": \":bold\"",
            "\"name\": \"variant\"",
            "\"argument\": \":wide\"",
            "(print (list (quote (wrap :style (:bold :wide))) :bold :wide t))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_unused_environment_parameter() {
    assert_common_lisp_inline_success(
        "(defmacro wrap (&environment env head &body body) `(list ,head ,@body))\n\
         (print (wrap :x 1 2 3))",
        &[
            "\"function_name\": \"wrap\"",
            "\"replacement\": \"(list :x 1 2 3)\"",
            "(print (list :x 1 2 3))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_required_destructuring() {
    assert_common_lisp_inline_success(
        "(defmacro swap ((left right)) `(list ,right ,left))\n\
         (print (swap (x y)))",
        &[
            "\"function_name\": \"swap\"",
            "\"replacement\": \"(list y x)\"",
            "\"name\": \"left\"",
            "\"argument\": \"x\"",
            "\"name\": \"right\"",
            "\"argument\": \"y\"",
            "(print (list y x))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_optional_destructuring() {
    assert_common_lisp_inline_success(
        "(defmacro swap (&optional ((left right) '(x y))) `(list ,right ,left))\n\
         (print (swap (a b)))",
        &[
            "\"function_name\": \"swap\"",
            "\"replacement\": \"(list b a)\"",
            "\"name\": \"left\"",
            "\"argument\": \"a\"",
            "\"name\": \"right\"",
            "\"argument\": \"b\"",
            "(print (list b a))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_inner_optional_destructuring() {
    assert_common_lisp_inline_success(
        "(defmacro wrap ((head &optional (tail head))) `(list ,head ,tail))\n\
         (print (wrap (a)))",
        &[
            "\"function_name\": \"wrap\"",
            "\"replacement\": \"(list a a)\"",
            "\"name\": \"head\"",
            "\"argument\": \"a\"",
            "\"name\": \"tail\"",
            "(print (list a a))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_top_level_key_destructuring() {
    assert_common_lisp_inline_success(
        "(defmacro wrap (&key ((:style (mode variant)))) `(list ,mode ,variant))\n\
         (print (wrap :style (:bold :wide)))",
        &[
            "\"function_name\": \"wrap\"",
            "\"replacement\": \"(list :bold :wide)\"",
            "\"name\": \"mode\"",
            "\"argument\": \":bold\"",
            "\"name\": \"variant\"",
            "\"argument\": \":wide\"",
            "(print (list :bold :wide))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_top_level_key_destructuring_default_value() {
    assert_common_lisp_inline_success(
        "(defmacro wrap (&key ((:style (mode variant)) '(:plain :narrow) style-p)) `(list ,mode ,variant ,style-p))\n\
         (print (wrap))",
        &[
            "\"function_name\": \"wrap\"",
            "\"replacement\": \"(list :plain :narrow nil)\"",
            "\"parameters\": []",
            "(print (list :plain :narrow nil))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_defmacro_top_level_key_destructuring_supplied_p() {
    assert_common_lisp_inline_success(
        "(defmacro wrap (&key ((:style (mode variant)) '(:plain :narrow) style-p)) `(list ,mode ,variant ,style-p))\n\
         (print (wrap :style (:bold :wide)))",
        &[
            "\"function_name\": \"wrap\"",
            "\"replacement\": \"(list :bold :wide t)\"",
            "\"name\": \"mode\"",
            "\"argument\": \":bold\"",
            "\"name\": \"variant\"",
            "\"argument\": \":wide\"",
            "(print (list :bold :wide t))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_aux_binding_when_duplicate_evaluation_is_allowed() {
    assert_common_lisp_inline_success_with_args(
        &["--allow-duplicate-evaluation"],
        "(defun render-one (x &aux (y x) (z y)) (list y z))\n\
         (print (render-one 1))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 1)\"",
            "\"name\": \"y\"",
            "\"argument\": \"x\"",
            "\"name\": \"z\"",
            "\"argument\": \"x\"",
            "(print (list 1 1))",
        ],
        &[],
    );
}

#[test]
fn cli_rejects_inline_function_when_common_lisp_environment_parameter_is_referenced() {
    assert_common_lisp_inline_failure(
        "(defmacro inspect (&environment env x) `(list ,env ,x))\n\
         (print (inspect 42))",
        &["reference &environment parameter 'env'"],
    );
}

#[test]
fn cli_rejects_inline_function_when_common_lisp_define_compiler_macro_environment_parameter_is_referenced()
 {
    assert_common_lisp_inline_failure(
        "(define-compiler-macro inspect (&environment env x) `(list ,env ,x))\n\
         (print (inspect 42))",
        &["reference &environment parameter 'env'"],
    );
}

#[test]
fn cli_rejects_inline_function_when_common_lisp_macro_duplicates_argument_evaluation() {
    assert_common_lisp_inline_failure(
        "(defmacro dup (x) `(+ ,x ,x))\n(print (dup (next)))",
        &["duplicate argument"],
    );
}

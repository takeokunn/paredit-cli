use super::*;

#[test]
fn cli_adds_common_lisp_required_parameter_before_optional_and_key_sections_when_requested() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "req",
            "--argument",
            "42",
            "--parameter-section",
            "positional",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node &optional stream &key color) (list node stream color req))\n(render item out :color :red)",
        &[
            "\"parameter_section\": \"required\"",
            "(defun render (node req &optional stream &key color)",
            "(render item 42 out :color :red)",
        ],
    );
}

#[test]
fn cli_adds_common_lisp_required_parameter_before_optional_section_when_requested() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "req",
            "--argument",
            "42",
            "--parameter-section",
            "positional",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node &optional stream) (list req node stream))\n(render item out)",
        &[
            "\"parameter_section\": \"required\"",
            "(defun render (node req &optional stream)",
            "(render item 42 out)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_key_parameter() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--argument",
            "8",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node &key color) (list node color margin))\n(render item :color :red)",
        &[
            "\"parameter_name\": \"margin\"",
            "(defun render (node &key color margin)",
            "(render item :color :red :margin 8)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_optional_parameter() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "style",
            "--argument",
            ":compact",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node &optional stream) (list node stream style))\n(render item out)",
        &[
            "\"parameter_name\": \"style\"",
            "(defun render (node &optional stream style)",
            "(render item out :compact)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_optional_before_key_parameter() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "style",
            "--argument",
            ":compact",
            "--call-path",
            "1",
            "--parameter-section",
            "optional",
            "--output",
            "json",
        ],
        "(defun render (node &optional stream &key color) (list node stream style color))\n(render item out :color :red)",
        &[
            "\"parameter_name\": \"style\"",
            "\"parameter_section\": \"optional\"",
            "(defun render (node &optional stream style &key color)",
            "(render item out :compact :color :red)",
        ],
    );
}

#[test]
fn cli_creates_common_lisp_optional_section_when_requested() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "style",
            "--argument",
            ":compact",
            "--parameter-section",
            "optional",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node) (list node style))\n(render item)",
        &[
            "\"parameter_section\": \"optional\"",
            "(defun render (node &optional style)",
            "(render item :compact)",
        ],
    );
}

#[test]
fn cli_creates_common_lisp_optional_section_before_existing_key_section() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "style",
            "--argument",
            ":compact",
            "--parameter-section",
            "optional",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node &key color) (list node style color))\n(render item :color :red)",
        &[
            "\"parameter_section\": \"optional\"",
            "(defun render (node &optional style &key color)",
            "(render item :compact :color :red)",
        ],
    );
}

#[test]
fn cli_creates_common_lisp_keyword_section_when_requested() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--argument",
            "8",
            "--parameter-section",
            "keyword",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node) (list node margin))\n(render item)",
        &[
            "\"parameter_section\": \"keyword\"",
            "(defun render (node &key margin)",
            "(render item :margin 8)",
        ],
    );
}

#[test]
fn cli_creates_common_lisp_keyword_section_after_existing_optional_section() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--argument",
            "8",
            "--parameter-section",
            "keyword",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node &optional stream) (list node stream margin))\n(render item out)",
        &[
            "\"parameter_section\": \"keyword\"",
            "(defun render (node &optional stream &key margin)",
            "(render item out :margin 8)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_before_allow_other_keys() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--argument",
            "8",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun render (node &key color &allow-other-keys) (list node color margin))\n(render item :color :red)",
        &[
            "\"parameter_section\": \"keyword\"",
            "(defun render (node &key color margin &allow-other-keys)",
            "(render item :color :red :margin 8)",
        ],
    );
}

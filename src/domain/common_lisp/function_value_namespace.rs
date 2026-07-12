use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::apply_reader_prefix_context;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

use super::{
    CommonLispLocalCallableForm, CommonLispValueScopeForm, common_lisp_local_callable_form,
    common_lisp_operator_head_eq, common_lisp_symbol_reference_eq,
    destructuring_lambda_list_bindings, macro_lambda_list_bindings, ordinary_lambda_list_bindings,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FunctionValueNamespaceDiagnostic {
    pub(crate) span: ByteSpan,
    pub(crate) name: String,
    binding_kind: LocalCallableKind,
}

impl FunctionValueNamespaceDiagnostic {
    pub(crate) fn code(&self) -> &'static str {
        match self.binding_kind {
            LocalCallableKind::Function => "function-used-as-value",
            LocalCallableKind::Macro => "macro-used-as-value",
        }
    }

    pub(crate) fn message(&self) -> String {
        match self.binding_kind {
            LocalCallableKind::Function => {
                format!(
                    "`{}` is a local function binding, not a value binding",
                    self.name
                )
            }
            LocalCallableKind::Macro => {
                format!(
                    "`{}` is a local macro binding, not a value binding",
                    self.name
                )
            }
        }
    }

    pub(crate) fn suggestion(&self) -> String {
        match self.binding_kind {
            LocalCallableKind::Function => format!(
                "pass the local function as `#'{}'`, or introduce a value binding named `{}`",
                self.name, self.name
            ),
            LocalCallableKind::Macro => format!(
                "a local macro cannot be passed to funcall/apply; invoke it in operator position, or introduce a function/value binding named `{}`",
                self.name
            ),
        }
    }
}

#[derive(Clone, Copy)]
struct NamespaceScope<'a> {
    local_callables: &'a [LocalCallableBinding],
    value_bindings: &'a [String],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LocalCallableKind {
    Function,
    Macro,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalCallableBinding {
    name: String,
    kind: LocalCallableKind,
}

pub(crate) fn function_value_namespace_diagnostics(
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<Vec<FunctionValueNamespaceDiagnostic>> {
    if !matches!(dialect, Dialect::CommonLisp | Dialect::Unknown) {
        return Ok(Vec::new());
    }
    let mut diagnostics = Vec::new();
    for root_index in 0..tree.root_view().children.len() {
        let path = Path::root_child(root_index);
        let view = tree.select_path(&path)?.view();
        collect(
            dialect,
            &view,
            &path,
            0,
            NamespaceScope {
                local_callables: &[],
                value_bindings: &[],
            },
            &mut diagnostics,
        );
    }
    Ok(diagnostics)
}

fn collect(
    dialect: Dialect,
    view: &ExpressionView,
    path: &Path,
    quasiquote_depth: usize,
    scope: NamespaceScope<'_>,
    diagnostics: &mut Vec<FunctionValueNamespaceDiagnostic>,
) {
    let Some(quasiquote_depth) = apply_reader_prefix_context(view, quasiquote_depth) else {
        return;
    };
    if let Some(head) = atom_child(view, 0) {
        if let Some(form) = common_lisp_local_callable_form(dialect, head) {
            collect_local_callable_form(
                dialect,
                view,
                path,
                quasiquote_depth,
                form,
                scope,
                diagnostics,
            );
            return;
        }
        match dialect.common_lisp_value_scope_form_for_head(head) {
            Some(CommonLispValueScopeForm::Let(_)) => {
                collect_let_form(dialect, view, path, quasiquote_depth, scope, diagnostics);
                return;
            }
            Some(CommonLispValueScopeForm::Lambda) => {
                collect_lambda_form(dialect, view, path, quasiquote_depth, scope, diagnostics);
                return;
            }
            Some(CommonLispValueScopeForm::Value) => {
                collect_destructuring_value_form(
                    dialect,
                    view,
                    path,
                    quasiquote_depth,
                    scope,
                    diagnostics,
                );
                return;
            }
            _ => {}
        }
    }
    if let Some(argument) = function_value_argument(view) {
        if let Some(binding) = callable_binding(scope.local_callables, argument)
            .filter(|_| !is_bound(scope.value_bindings, argument))
        {
            diagnostics.push(FunctionValueNamespaceDiagnostic {
                span: view.children[1].span,
                name: argument.to_owned(),
                binding_kind: binding.kind,
            });
        }
    }
    collect_children(dialect, view, path, quasiquote_depth, scope, diagnostics);
}

fn collect_local_callable_form(
    dialect: Dialect,
    view: &ExpressionView,
    path: &Path,
    quasiquote_depth: usize,
    form: CommonLispLocalCallableForm,
    scope: NamespaceScope<'_>,
    diagnostics: &mut Vec<FunctionValueNamespaceDiagnostic>,
) {
    let body_callables = callable_body_scope(scope.local_callables, view, form);
    let binding_callables =
        callable_binding_body_scope(form, scope.local_callables, &body_callables);
    if let Some(bindings) = view.children.get(1) {
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            let mut binding_values = scope.value_bindings.to_vec();
            binding_values.extend(callable_lambda_list_bindings(form, binding));
            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                collect(
                    dialect,
                    child,
                    &path.descendant([1, binding_index, child_index]),
                    quasiquote_depth,
                    NamespaceScope {
                        local_callables: binding_callables,
                        value_bindings: &binding_values,
                    },
                    diagnostics,
                );
            }
        }
    }
    collect_body(
        dialect,
        view,
        path,
        quasiquote_depth,
        NamespaceScope {
            local_callables: &body_callables,
            value_bindings: scope.value_bindings,
        },
        diagnostics,
    );
}

fn collect_let_form(
    dialect: Dialect,
    view: &ExpressionView,
    path: &Path,
    quasiquote_depth: usize,
    scope: NamespaceScope<'_>,
    diagnostics: &mut Vec<FunctionValueNamespaceDiagnostic>,
) {
    let Some(head) = atom_child(view, 0) else {
        return;
    };
    let Some(CommonLispValueScopeForm::Let(form)) =
        dialect.common_lisp_value_scope_form_for_head(head)
    else {
        return;
    };
    let mut body_bindings = scope.value_bindings.to_vec();
    let mut initializer_bindings = scope.value_bindings.to_vec();
    if let Some(bindings) = view.children.get(1) {
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            for (child_index, child) in binding.children.iter().enumerate().skip(1) {
                collect(
                    dialect,
                    child,
                    &path.descendant([1, binding_index, child_index]),
                    quasiquote_depth,
                    NamespaceScope {
                        local_callables: scope.local_callables,
                        value_bindings: &initializer_bindings,
                    },
                    diagnostics,
                );
            }
            if let Some(name) = atom_child(binding, 0) {
                body_bindings.push(name.to_owned());
                if form.is_sequential() {
                    initializer_bindings.push(name.to_owned());
                }
            }
        }
    }
    collect_body(
        dialect,
        view,
        path,
        quasiquote_depth,
        NamespaceScope {
            local_callables: scope.local_callables,
            value_bindings: &body_bindings,
        },
        diagnostics,
    );
}

fn collect_lambda_form(
    dialect: Dialect,
    view: &ExpressionView,
    path: &Path,
    quasiquote_depth: usize,
    scope: NamespaceScope<'_>,
    diagnostics: &mut Vec<FunctionValueNamespaceDiagnostic>,
) {
    let mut body_bindings = scope.value_bindings.to_vec();
    body_bindings.extend(ordinary_lambda_list_bindings(view));
    collect_body(
        dialect,
        view,
        path,
        quasiquote_depth,
        NamespaceScope {
            local_callables: scope.local_callables,
            value_bindings: &body_bindings,
        },
        diagnostics,
    );
}

fn collect_destructuring_value_form(
    dialect: Dialect,
    view: &ExpressionView,
    path: &Path,
    quasiquote_depth: usize,
    scope: NamespaceScope<'_>,
    diagnostics: &mut Vec<FunctionValueNamespaceDiagnostic>,
) {
    if let Some(value_form) = view.children.get(2) {
        collect(
            dialect,
            value_form,
            &path.child(2),
            quasiquote_depth,
            scope,
            diagnostics,
        );
    }
    let mut body_bindings = scope.value_bindings.to_vec();
    if let Some(pattern) = view.children.get(1) {
        body_bindings.extend(destructuring_lambda_list_bindings(pattern));
    }
    for (child_index, child) in view.children.iter().enumerate().skip(3) {
        collect(
            dialect,
            child,
            &path.child(child_index),
            quasiquote_depth,
            NamespaceScope {
                local_callables: scope.local_callables,
                value_bindings: &body_bindings,
            },
            diagnostics,
        );
    }
}

fn collect_body(
    dialect: Dialect,
    view: &ExpressionView,
    path: &Path,
    quasiquote_depth: usize,
    scope: NamespaceScope<'_>,
    diagnostics: &mut Vec<FunctionValueNamespaceDiagnostic>,
) {
    for (child_index, child) in view.children.iter().enumerate().skip(2) {
        collect(
            dialect,
            child,
            &path.child(child_index),
            quasiquote_depth,
            scope,
            diagnostics,
        );
    }
}

fn collect_children(
    dialect: Dialect,
    view: &ExpressionView,
    path: &Path,
    quasiquote_depth: usize,
    scope: NamespaceScope<'_>,
    diagnostics: &mut Vec<FunctionValueNamespaceDiagnostic>,
) {
    for (child_index, child) in view.children.iter().enumerate() {
        collect(
            dialect,
            child,
            &path.child(child_index),
            quasiquote_depth,
            scope,
            diagnostics,
        );
    }
}

fn function_value_argument(view: &ExpressionView) -> Option<&str> {
    let head = atom_child(view, 0)?;
    let argument = view.children.get(1)?;
    (is_function_value_consumer(head) && argument.reader_prefixes.is_empty())
        .then(|| atom_text(argument))
        .flatten()
}

fn is_function_value_consumer(head: &str) -> bool {
    common_lisp_operator_head_eq(head, "funcall") || common_lisp_operator_head_eq(head, "apply")
}

fn callable_body_scope(
    outer: &[LocalCallableBinding],
    view: &ExpressionView,
    form: CommonLispLocalCallableForm,
) -> Vec<LocalCallableBinding> {
    let mut bindings = outer.to_vec();
    let kind = if form.is_macro() {
        LocalCallableKind::Macro
    } else {
        LocalCallableKind::Function
    };
    if let Some(local_bindings) = view.children.get(1) {
        for binding in &local_bindings.children {
            if let Some(name) = atom_child(binding, 0) {
                bindings.push(LocalCallableBinding {
                    name: name.to_owned(),
                    kind,
                });
            }
        }
    }
    bindings
}

fn callable_binding_body_scope<'a>(
    form: CommonLispLocalCallableForm,
    outer: &'a [LocalCallableBinding],
    body: &'a [LocalCallableBinding],
) -> &'a [LocalCallableBinding] {
    if form == CommonLispLocalCallableForm::Labels {
        body
    } else {
        outer
    }
}

fn callable_lambda_list_bindings(
    form: CommonLispLocalCallableForm,
    binding: &ExpressionView,
) -> Vec<String> {
    if form.is_macro() {
        macro_lambda_list_bindings(binding)
    } else {
        ordinary_lambda_list_bindings(binding)
    }
}

fn callable_binding<'a>(
    bindings: &'a [LocalCallableBinding],
    name: &str,
) -> Option<&'a LocalCallableBinding> {
    bindings
        .iter()
        .rev()
        .find(|binding| common_lisp_symbol_reference_eq(&binding.name, name))
}

fn is_bound(scope: &[String], name: &str) -> bool {
    scope
        .iter()
        .any(|candidate| common_lisp_symbol_reference_eq(candidate, name))
}

fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

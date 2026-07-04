use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::list_edit::{atom_child, atom_text};

#[derive(Debug)]
pub(super) struct FunctionParameterTarget {
    pub(super) function_name: SymbolName,
    pub(super) parameter_container: ExpressionView,
    pub(super) protected_prefix_count: usize,
    pub(super) definition_span: ByteSpan,
}

pub(super) fn parse_remove_function_parameter_definition(
    dialect: Dialect,
    view: ExpressionView,
) -> Result<FunctionParameterTarget> {
    parse_function_parameter_definition(dialect, view, None, "remove-function-parameter")
}

pub(super) fn parse_move_function_parameter_definition(
    dialect: Dialect,
    view: ExpressionView,
) -> Result<FunctionParameterTarget> {
    parse_function_parameter_definition(dialect, view, None, "move-function-parameter")
}

pub(super) fn parse_add_function_parameter_definition(
    dialect: Dialect,
    view: ExpressionView,
    new_parameter: &SymbolName,
) -> Result<FunctionParameterTarget> {
    parse_function_parameter_definition(
        dialect,
        view,
        Some(new_parameter),
        "add-function-parameter",
    )
}

fn parse_function_parameter_definition(
    dialect: Dialect,
    view: ExpressionView,
    new_parameter: Option<&SymbolName>,
    operation: &str,
) -> Result<FunctionParameterTarget> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        anyhow::bail!("{operation} definition selection must be a function definition list");
    }
    if view.children.len() < 3 {
        anyhow::bail!("{operation} definition must include a name and parameters");
    }

    let head = atom_text(&view.children[0])
        .with_context(|| format!("{operation} definition must start with a definition atom"))?;
    if !function_head_supported(dialect, head) {
        anyhow::bail!("{operation} does not support definition head: {head}");
    }

    let (function_name, parameter_container, protected_prefix_count, existing_parameters) = if head
        == "define"
    {
        let signature = view.children.get(1).context(
            "scheme define selection must include a signature list: (define (name args...) body)",
        )?;
        if signature.kind != ExpressionKind::List || signature.delimiter != Some(Delimiter::Paren) {
            anyhow::bail!(
                "{operation} currently supports scheme procedure defines, not variable defines"
            );
        }
        let name = atom_child(signature, 0)
            .context("scheme define signature must start with a function name")?;
        (
            SymbolName::new(name.to_owned())?,
            signature.clone(),
            1,
            parameter_names_from_children(&signature.children[1..])?,
        )
    } else {
        let name =
            atom_child(&view, 1).context("function definition must include a symbol name")?;
        let params = view
            .children
            .get(2)
            .context("function definition must include a parameter list")?;
        (
            SymbolName::new(name.to_owned())?,
            params.clone(),
            0,
            parameter_names(params)?,
        )
    };

    if let Some(new_parameter) = new_parameter {
        if existing_parameters
            .iter()
            .any(|parameter| parameter == new_parameter.as_str())
        {
            anyhow::bail!(
                "add-function-parameter parameter '{}' already exists in {}",
                new_parameter,
                function_name
            );
        }
    }

    Ok(FunctionParameterTarget {
        function_name,
        parameter_container,
        protected_prefix_count,
        definition_span: view.span,
    })
}

fn function_head_supported(dialect: Dialect, head: &str) -> bool {
    match dialect {
        Dialect::CommonLisp | Dialect::EmacsLisp => matches!(head, "defun" | "defmacro"),
        Dialect::Scheme => matches!(head, "define"),
        Dialect::Clojure => matches!(head, "defn" | "defmacro"),
        Dialect::Janet => matches!(head, "defn" | "defmacro"),
        Dialect::Fennel => matches!(head, "fn" | "lambda"),
        Dialect::Unknown => matches!(
            head,
            "defun" | "defmacro" | "define" | "defn" | "fn" | "lambda"
        ),
    }
}

fn parameter_names(parameter_form: &ExpressionView) -> Result<Vec<String>> {
    match parameter_form.kind {
        ExpressionKind::List => parameter_names_from_children(&parameter_form.children),
        _ => anyhow::bail!("function parameter form must be a list or vector"),
    }
}

fn parameter_names_from_children(children: &[ExpressionView]) -> Result<Vec<String>> {
    let mut names = Vec::with_capacity(children.len());
    for child in children {
        let name = atom_text(child).context("function parameters must be atoms")?;
        if name.starts_with('&') {
            anyhow::bail!("function parameter modifiers are not supported yet: {name}");
        }
        names.push(name.to_owned());
    }
    Ok(names)
}

pub(super) fn find_unique_parameter_item_index(
    parameter_container: &ExpressionView,
    protected_prefix_count: usize,
    parameter_name: &SymbolName,
    operation: &str,
) -> Result<usize> {
    let mut found = None;
    for (index, child) in parameter_container
        .children
        .iter()
        .enumerate()
        .skip(protected_prefix_count)
    {
        let name = atom_text(child).with_context(|| {
            format!("{operation} currently supports only simple symbol parameters")
        })?;
        if name.starts_with('&') {
            anyhow::bail!("{operation} does not support lambda-list keyword parameter yet: {name}");
        }
        SymbolName::new(name.to_owned())?;
        if name == parameter_name.as_str() && found.replace(index).is_some() {
            anyhow::bail!(
                "{operation} parameter '{}' appears more than once",
                parameter_name
            );
        }
    }

    found.with_context(|| format!("{operation} parameter '{}' was not found", parameter_name))
}

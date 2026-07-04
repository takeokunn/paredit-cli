use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::syntax::{atom_child, atom_text};

pub(super) fn parse_inline_function_definition(
    dialect: Dialect,
    view: ExpressionView,
) -> Result<(SymbolName, Vec<String>, ExpressionView)> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        anyhow::bail!("inline-function definition selection must be a function definition list");
    }
    if view.children.len() < 3 {
        anyhow::bail!(
            "inline-function definition must include a name, parameters, and one body expression"
        );
    }

    let head = atom_text(&view.children[0])
        .context("inline-function definition must start with a definition atom")?;
    if !inline_function_head_supported(dialect, head) {
        anyhow::bail!("inline-function does not support definition head: {head}");
    }

    let (name, params, body_start) = if head == "define" {
        let signature = view.children.get(1).context(
            "scheme define selection must include a signature list: (define (name args...) body)",
        )?;
        if signature.kind != ExpressionKind::List || signature.delimiter != Some(Delimiter::Paren) {
            anyhow::bail!(
                "inline-function currently supports scheme procedure defines, not variable defines"
            );
        }
        let name = atom_child(signature, 0)
            .context("scheme define signature must start with a function name")?;
        (
            SymbolName::new(name.to_owned())?,
            inline_parameter_names_from_children(&signature.children[1..])?,
            2,
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
            inline_parameter_names(params)?,
            3,
        )
    };

    let body_forms = &view.children[body_start..];
    let [body] = body_forms else {
        anyhow::bail!(
            "inline-function currently requires exactly one body expression; found {}",
            body_forms.len()
        );
    };

    Ok((name, params, body.clone()))
}

fn inline_function_head_supported(dialect: Dialect, head: &str) -> bool {
    let normalized = head
        .trim_start_matches("cl:")
        .trim_start_matches("cl-user:");
    match normalized {
        "defun" | "cl-defun" | "defsubst" | "definline" | "defn" | "defn-" => true,
        "define" => matches!(dialect, Dialect::Scheme | Dialect::Unknown),
        "fn" => matches!(dialect, Dialect::Fennel),
        _ => false,
    }
}

fn inline_parameter_names(parameter_form: &ExpressionView) -> Result<Vec<String>> {
    match parameter_form.delimiter {
        Some(Delimiter::Paren | Delimiter::Bracket) => {
            inline_parameter_names_from_children(&parameter_form.children)
        }
        _ => anyhow::bail!("inline-function currently supports only flat symbol parameter lists"),
    }
}

fn inline_parameter_names_from_children(children: &[ExpressionView]) -> Result<Vec<String>> {
    let mut params = Vec::with_capacity(children.len());
    for child in children {
        let name = atom_text(child)
            .context("inline-function currently supports only simple symbol parameters")?;
        if name.starts_with('&') {
            anyhow::bail!(
                "inline-function does not support lambda-list keyword parameter yet: {name}"
            );
        }
        SymbolName::new(name.to_owned())?;
        params.push(name.to_owned());
    }
    Ok(params)
}

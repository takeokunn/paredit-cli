use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::syntax::{atom_child, atom_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InlineParameter {
    pub name: String,
    pub keyword: Option<String>,
}

pub(super) fn parse_inline_function_definition(
    dialect: Dialect,
    view: ExpressionView,
) -> Result<(SymbolName, Vec<InlineParameter>, ExpressionView)> {
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
            inline_parameter_names_from_children(dialect, &signature.children[1..])?,
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
            inline_parameter_names(dialect, params)?,
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

fn inline_parameter_names(
    dialect: Dialect,
    parameter_form: &ExpressionView,
) -> Result<Vec<InlineParameter>> {
    match parameter_form.delimiter {
        Some(Delimiter::Paren | Delimiter::Bracket) => {
            inline_parameter_names_from_children(dialect, &parameter_form.children)
        }
        _ => anyhow::bail!("inline-function currently supports only flat symbol parameter lists"),
    }
}

fn inline_parameter_names_from_children(
    dialect: Dialect,
    children: &[ExpressionView],
) -> Result<Vec<InlineParameter>> {
    let mut params = Vec::with_capacity(children.len());
    let mut optional = false;
    let mut keyword = false;
    let mut allow_other_keys = false;
    let supports_common_lisp_lambda_list = matches!(
        dialect,
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Unknown
    );

    for child in children {
        if let Some(marker) = atom_text(child).filter(|name| name.starts_with('&')) {
            if !supports_common_lisp_lambda_list {
                anyhow::bail!(
                    "inline-function function parameter modifiers are not supported: {marker}"
                );
            }
            match marker {
                "&optional" => {
                    optional = true;
                    keyword = false;
                    allow_other_keys = false;
                }
                "&key" => {
                    keyword = true;
                    optional = false;
                    allow_other_keys = false;
                }
                "&allow-other-keys" if keyword => allow_other_keys = true,
                _ => anyhow::bail!(
                    "inline-function currently supports only required, &optional, and simple &key parameters; found {marker}"
                ),
            }
            continue;
        }

        if allow_other_keys {
            anyhow::bail!(
                "inline-function does not support &key parameters after &allow-other-keys"
            );
        }

        let param = if keyword {
            keyword_parameter(child)?
        } else if optional {
            InlineParameter {
                name: optional_parameter_name(child)?.to_owned(),
                keyword: None,
            }
        } else {
            let name = atom_text(child)
                .context("inline-function currently supports only simple symbol parameters")?;
            InlineParameter {
                name: name.to_owned(),
                keyword: None,
            }
        };
        SymbolName::new(param.name.clone())?;
        params.push(param);
    }
    Ok(params)
}

fn optional_parameter_name(child: &ExpressionView) -> Result<&str> {
    if let Some(name) = atom_text(child) {
        return Ok(name);
    }

    let binding = child.children.first().context(
        "inline-function currently supports only simple &optional parameter specifications",
    )?;
    if child.children.len() > 2 {
        anyhow::bail!("inline-function does not support &optional supplied-p parameters yet");
    }
    atom_text(binding).context(
        "inline-function currently supports only simple &optional parameter specifications",
    )
}

fn keyword_parameter(child: &ExpressionView) -> Result<InlineParameter> {
    if let Some(name) = atom_text(child) {
        if name.starts_with(':') {
            anyhow::bail!("inline-function requires a binding name for &key parameter {name}");
        }
        return Ok(InlineParameter {
            name: name.to_owned(),
            keyword: Some(format!(":{name}")),
        });
    }

    if child.children.len() > 2 {
        anyhow::bail!("inline-function does not support &key supplied-p parameters yet");
    }

    let binding = child
        .children
        .first()
        .context("inline-function currently supports only simple &key parameter specifications")?;
    if let Some(name) = atom_text(binding) {
        if name.starts_with(':') {
            anyhow::bail!("inline-function requires a binding name for &key parameter {name}");
        }
        return Ok(InlineParameter {
            name: name.to_owned(),
            keyword: Some(format!(":{name}")),
        });
    }

    let [external, internal] = binding.children.as_slice() else {
        anyhow::bail!("inline-function currently supports only (:keyword name) &key bindings");
    };
    let external = atom_text(external)
        .context("inline-function currently supports only atom &key external names")?;
    if !external.starts_with(':') {
        anyhow::bail!("inline-function &key external name must be a keyword: {external}");
    }
    let internal = atom_text(internal)
        .context("inline-function currently supports only atom &key internal names")?;
    if internal.starts_with(':') {
        anyhow::bail!("inline-function &key internal binding must not be a keyword: {internal}");
    }

    Ok(InlineParameter {
        name: internal.to_owned(),
        keyword: Some(external.to_owned()),
    })
}

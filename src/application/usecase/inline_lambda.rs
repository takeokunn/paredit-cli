//! Use case for safely replacing an immediately invoked Common Lisp lambda with `let`.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

#[derive(Debug, Clone)]
pub struct InlineLambdaRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct InlineLambdaBindingPlan {
    pub name: SymbolName,
    pub argument: String,
}

#[derive(Debug, Clone)]
pub struct InlineLambdaPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub call_span: ByteSpan,
    pub lambda_span: ByteSpan,
    pub bindings: Vec<InlineLambdaBindingPlan>,
    pub replacement: String,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_inline_lambda(request: InlineLambdaRequest<'_>) -> Result<InlineLambdaPlan> {
    if request.dialect != Dialect::CommonLisp {
        bail!("inline-lambda currently supports only Common Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("inline-lambda input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let selection = tree.select_path(&request.path)?;
    let call = selection.view();
    if tree.has_comment_in(call.span) {
        bail!("inline-lambda cannot replace a call containing comments");
    }
    if call.kind != ExpressionKind::List || !call.reader_prefixes.is_empty() {
        bail!("inline-lambda selected form must be a plain call list");
    }
    let lambda = call
        .children
        .first()
        .context("inline-lambda selected call has no operator")?;
    require_list_head(lambda, "lambda", "call operator must be a lambda form")?;
    if lambda.children.len() != 3 {
        bail!("inline-lambda requires exactly one lambda body expression");
    }

    let parameter_list = &lambda.children[1];
    if parameter_list.kind != ExpressionKind::List || !parameter_list.reader_prefixes.is_empty() {
        bail!("inline-lambda requires a plain required-parameter list");
    }
    let mut parameter_names = Vec::with_capacity(parameter_list.children.len());
    for parameter in &parameter_list.children {
        let name = plain_symbol(parameter, "required parameter")?;
        if name.as_str().starts_with('&') {
            bail!("inline-lambda supports required parameters only");
        }
        if parameter_names.iter().any(|existing: &SymbolName| {
            common_lisp_symbol_reference_eq(existing.as_str(), name.as_str())
        }) {
            bail!("inline-lambda requires unique parameter names");
        }
        parameter_names.push(name);
    }
    if call.children.len() != parameter_names.len() + 1 {
        bail!("inline-lambda requires exact call arity");
    }

    let body = &lambda.children[2];
    reject_function_boundary_forms(body)?;
    let bindings = parameter_names
        .into_iter()
        .zip(&call.children[1..])
        .map(|(name, argument)| InlineLambdaBindingPlan {
            name,
            argument: argument.span.slice(request.input).to_owned(),
        })
        .collect::<Vec<_>>();
    let rendered_bindings = bindings
        .iter()
        .map(|binding| format!("({} {})", binding.name, binding.argument))
        .collect::<Vec<_>>()
        .join(" ");
    let replacement = format!(
        "(let ({rendered_bindings}) {})",
        body.span.slice(request.input)
    );
    let rewritten = replace_span(request.input, call.span, &replacement);
    SyntaxTree::parse(&rewritten)
        .context("inline-lambda output is not a valid S-expression document")?;

    Ok(InlineLambdaPlan {
        dialect: request.dialect,
        path: request.path,
        call_span: call.span,
        lambda_span: lambda.span,
        bindings,
        replacement,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn plain_symbol(view: &ExpressionView, role: &str) -> Result<SymbolName> {
    if view.kind != ExpressionKind::Atom || !view.reader_prefixes.is_empty() {
        bail!("inline-lambda requires a plain {role}");
    }
    let text =
        atom_symbol_text(view).with_context(|| format!("inline-lambda requires a plain {role}"))?;
    SymbolName::new(text).with_context(|| format!("inline-lambda has invalid {role}"))
}

fn require_list_head(view: &ExpressionView, expected: &str, message: &str) -> Result<()> {
    let matches = view.kind == ExpressionKind::List
        && view.reader_prefixes.is_empty()
        && view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| common_lisp_symbol_reference_eq(head, expected));
    if !matches {
        bail!("inline-lambda {message}");
    }
    Ok(())
}

fn reject_function_boundary_forms(view: &ExpressionView) -> Result<()> {
    if view.kind == ExpressionKind::List
        && view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| {
                ["go", "return", "return-from", "declare"]
                    .iter()
                    .any(|form| common_lisp_symbol_reference_eq(head, form))
            })
    {
        bail!("inline-lambda rejects control transfer or declarations tied to a function boundary");
    }
    for child in &view.children {
        reject_function_boundary_forms(child)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(input: &str) -> Result<InlineLambdaPlan> {
        plan_inline_lambda(InlineLambdaRequest {
            input,
            dialect: Dialect::CommonLisp,
            path: Path::from_indexes(vec![0]),
        })
    }

    #[test]
    fn replaces_immediate_lambda_call_with_parallel_let() {
        let plan = plan("((lambda (x y) (+ x y)) (next-x) (next-y))").unwrap();
        assert_eq!(plan.rewritten, "(let ((x (next-x)) (y (next-y))) (+ x y))");
        assert_eq!(plan.bindings.len(), 2);
    }

    #[test]
    fn preserves_argument_evaluation_even_when_parameter_is_unused_or_repeated() {
        assert_eq!(
            plan("((lambda (x) 1) (effect))").unwrap().rewritten,
            "(let ((x (effect))) 1)"
        );
        assert_eq!(
            plan("((lambda (x) (+ x x)) (effect))").unwrap().rewritten,
            "(let ((x (effect))) (+ x x))"
        );
    }

    #[test]
    fn rejects_extended_lambda_lists_wrong_arity_and_function_boundary_forms() {
        assert!(plan("((lambda (&optional x) x) 1)").is_err());
        assert!(plan("((lambda (x) x) 1 2)").is_err());
        assert!(plan("((lambda () (return 1)))").is_err());
        assert!(plan("((lambda () (return-from nil 1)))").is_err());
        assert!(plan("((lambda () (declare (optimize speed))))").is_err());
    }
}

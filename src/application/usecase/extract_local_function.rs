//! Use case for extracting an expression into an enclosing local function.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_function::{
    infer_extract_function_params, rewrite::extracted_call,
};
use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::{
    common_lisp_local_callable_form, common_lisp_symbol_reference_eq, local_callable_names,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, Path, Selection, SymbolName, SyntaxTree,
};

#[derive(Debug, Clone)]
pub struct ExtractLocalFunctionRequest<'a> {
    pub input: &'a str,
    pub selection: Selection<'a>,
    pub path: Option<Path>,
    pub enclosing: Selection<'a>,
    pub enclosing_path: Path,
    pub dialect: Dialect,
    pub name: SymbolName,
    pub explicit_params: Vec<String>,
    pub infer_params: bool,
    pub recursive: bool,
}

#[derive(Debug, Clone)]
pub struct ExtractLocalFunctionPlan {
    pub path: Option<Path>,
    pub enclosing_path: Path,
    pub selected_span: ByteSpan,
    pub enclosing_span: ByteSpan,
    pub name: SymbolName,
    pub params: Vec<String>,
    pub inferred_params: Vec<String>,
    pub recursive: bool,
    pub call: String,
    pub replacement: String,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_extract_local_function(
    request: ExtractLocalFunctionRequest<'_>,
) -> Result<ExtractLocalFunctionPlan> {
    if request.dialect != Dialect::CommonLisp {
        bail!("extract-local-function currently supports only Common Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("extract-local-function input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let path = request
        .path
        .as_ref()
        .context("extract-local-function requires a path selection")?;
    if tree.select_path(path)?.span() != request.selection.span()
        || tree.select_path(&request.enclosing_path)?.span() != request.enclosing.span()
    {
        bail!("extract-local-function paths and selections must refer to the input tree");
    }
    reject_structural_position(&tree, path)?;

    let selected_span = request.selection.span();
    let enclosing_span = request.enclosing.span();
    let enclosing_view = request.enclosing.view();
    if enclosing_view.kind != ExpressionKind::List {
        bail!("extract-local-function enclosing selection must be a list");
    }
    if selected_span == enclosing_span
        || selected_span.start() < enclosing_span.start()
        || selected_span.end() > enclosing_span.end()
    {
        bail!("extract-local-function target must be a proper descendant of the enclosing list");
    }
    reject_local_name_collision(&enclosing_view, request.name.as_str())?;
    reject_existing_call_capture(
        &enclosing_view,
        request.name.as_str(),
        request.selection.span(),
    )?;
    reject_non_local_control_transfer(&request.selection.view())?;

    let mut params = request.explicit_params;
    let inferred_params = if request.infer_params {
        infer_extract_function_params(request.dialect, &request.selection.view(), &params)
    } else {
        Vec::new()
    };
    for param in &inferred_params {
        if !params
            .iter()
            .any(|existing| common_lisp_symbol_reference_eq(existing, param))
        {
            params.push(param.clone());
        }
    }

    let call = extracted_call(&request.name, &params);
    let selected = selected_span.slice(request.input);
    let enclosed = replace_within(request.input, enclosing_span, selected_span, &call);
    let operator = if request.recursive { "labels" } else { "flet" };
    let replacement = format!(
        "({operator} (({} ({}) {})) {})",
        request.name.as_str(),
        params.join(" "),
        selected,
        enclosed
    );
    let rewritten = replace_span(request.input, enclosing_span, &replacement);
    SyntaxTree::parse(&rewritten)
        .context("extracted local function output is not a valid S-expression document")?;

    Ok(ExtractLocalFunctionPlan {
        path: request.path,
        enclosing_path: request.enclosing_path,
        selected_span,
        enclosing_span,
        name: request.name,
        params,
        inferred_params,
        recursive: request.recursive,
        call,
        replacement,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn replace_within(input: &str, container: ByteSpan, target: ByteSpan, replacement: &str) -> String {
    let start = target.start().get() - container.start().get();
    let end = target.end().get() - container.start().get();
    let mut output = container.slice(input).to_owned();
    output.replace_range(start..end, replacement);
    output
}

fn reject_structural_position(tree: &SyntaxTree, path: &Path) -> Result<()> {
    let indexes = path.to_raw_indexes();
    for depth in 1..indexes.len() {
        let child_index = indexes[depth];
        if child_index == 0 {
            bail!("extract-local-function target cannot be in a list head position");
        }
        let parent_path = Path::from_indexes(indexes[..depth].to_vec());
        let parent = tree.select_path(&parent_path)?.view();
        let Some(head) = parent
            .children
            .first()
            .and_then(|child| child.text.as_deref())
        else {
            continue;
        };
        let structural_child = (common_lisp_symbol_reference_eq(head, "lambda")
            && child_index == 1)
            || (["defun", "defmacro", "defmethod"]
                .iter()
                .any(|form| common_lisp_symbol_reference_eq(head, form))
                && child_index == 2)
            || ([
                "let",
                "let*",
                "symbol-macrolet",
                "flet",
                "labels",
                "macrolet",
            ]
            .iter()
            .any(|form| common_lisp_symbol_reference_eq(head, form))
                && child_index == 1);
        if structural_child {
            bail!("extract-local-function target cannot be inside a structural binding position");
        }
    }
    Ok(())
}

fn reject_local_name_collision(view: &ExpressionView, name: &str) -> Result<()> {
    if let Some(head) = view
        .children
        .first()
        .and_then(|child| child.text.as_deref())
        && common_lisp_local_callable_form(Dialect::CommonLisp, head).is_some()
        && local_callable_names(view)
            .iter()
            .any(|bound| common_lisp_symbol_reference_eq(bound, name))
    {
        bail!("local function name '{name}' is already bound inside the enclosing list");
    }
    for child in &view.children {
        reject_local_name_collision(child, name)?;
    }
    Ok(())
}

fn reject_existing_call_capture(
    view: &ExpressionView,
    name: &str,
    selected: ByteSpan,
) -> Result<()> {
    // Calls inside the extracted expression move into the definition body. In
    // `labels` they intentionally become recursive; in `flet` they retain the
    // surrounding function binding. Only calls left in the wrapped body can be
    // captured by the newly introduced binding.
    if view.span == selected {
        return Ok(());
    }
    if view.kind == ExpressionKind::List
        && view
            .children
            .first()
            .and_then(|head| head.text.as_deref())
            .is_some_and(|head| common_lisp_symbol_reference_eq(head, name))
    {
        bail!("local function name '{name}' would capture an existing call in the enclosing list");
    }
    for child in &view.children {
        reject_existing_call_capture(child, name, selected)?;
    }
    Ok(())
}

fn reject_non_local_control_transfer(view: &ExpressionView) -> Result<()> {
    if view.kind == ExpressionKind::List
        && let Some(head) = view
            .children
            .first()
            .and_then(|child| child.text.as_deref())
        && (common_lisp_symbol_reference_eq(head, "go")
            || common_lisp_symbol_reference_eq(head, "return-from"))
    {
        bail!("extract-local-function cannot move {head} across a function boundary");
    }
    for child in &view.children {
        reject_non_local_control_transfer(child)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(
        input: &str,
        target: &Path,
        enclosing: &Path,
        recursive: bool,
    ) -> Result<ExtractLocalFunctionPlan> {
        let tree = SyntaxTree::parse(input)?;
        plan_extract_local_function(ExtractLocalFunctionRequest {
            input,
            selection: tree.select_path(target)?,
            path: Some(target.clone()),
            enclosing: tree.select_path(enclosing)?,
            enclosing_path: enclosing.clone(),
            dialect: Dialect::CommonLisp,
            name: SymbolName::new("compute")?,
            explicit_params: Vec::new(),
            infer_params: true,
            recursive,
        })
    }

    #[test]
    fn extracts_into_flet_and_infers_free_values() {
        let result = plan(
            "(defun render (x) (print (+ x 1)))",
            &Path::from_indexes(vec![0, 3, 1]),
            &Path::from_indexes(vec![0, 3]),
            false,
        )
        .expect("plan");
        assert_eq!(result.params, vec!["x"]);
        assert_eq!(
            result.rewritten,
            "(defun render (x) (flet ((compute (x) (+ x 1))) (print (compute x))))"
        );
    }

    #[test]
    fn recursive_uses_labels() {
        let result = plan(
            "(defun render () (print (+ 1 2)))",
            &Path::from_indexes(vec![0, 3, 1]),
            &Path::from_indexes(vec![0, 3]),
            true,
        )
        .expect("plan");
        assert!(result.rewritten.contains("(labels ((compute () (+ 1 2)))"));
    }

    #[test]
    fn recursive_allows_self_calls_inside_the_extracted_body() {
        let result = plan(
            "(defun render (x) (print (if (zerop x) 0 (compute (- x 1)))))",
            &Path::from_indexes(vec![0, 3, 1]),
            &Path::from_indexes(vec![0, 3]),
            true,
        )
        .expect("plan");
        assert!(result.rewritten.contains("(labels ((compute"));
        assert!(result.rewritten.contains("(compute (- x 1))"));
    }

    #[test]
    fn rejects_existing_local_binding_name() {
        let error = plan(
            "(defun render () (flet ((compute () 1)) (print (+ 1 2))))",
            &Path::from_indexes(vec![0, 3, 2, 1]),
            &Path::from_indexes(vec![0, 3]),
            false,
        )
        .expect_err("collision");
        assert!(error.to_string().contains("already bound"));
    }

    #[test]
    fn rejects_capture_of_existing_same_name_call() {
        let error = plan(
            "(defun render (x) (progn (compute 1) (+ x 1)))",
            &Path::from_indexes(vec![0, 3, 2]),
            &Path::from_indexes(vec![0, 3]),
            false,
        )
        .expect_err("captured call");
        assert!(error.to_string().contains("capture an existing call"));
    }

    #[test]
    fn rejects_non_local_control_transfer() {
        let error = plan(
            "(defun render () (block done (return-from done 1)))",
            &Path::from_indexes(vec![0, 3, 2]),
            &Path::from_indexes(vec![0, 3]),
            false,
        )
        .expect_err("return-from");
        assert!(error.to_string().contains("function boundary"));
    }

    #[test]
    fn rejects_lambda_list_position() {
        let error = plan(
            "(defun render (x) (print x))",
            &Path::from_indexes(vec![0, 2, 0]),
            &Path::from_indexes(vec![0, 2]),
            false,
        )
        .expect_err("lambda list");
        assert!(error.to_string().contains("structural binding position"));
    }

    #[test]
    fn generated_binding_is_not_swallowed_by_a_line_comment() {
        let result = plan(
            "(defun render () (progn ; retain comment\n  (+ 1 2)))",
            &Path::from_indexes(vec![0, 3, 1]),
            &Path::from_indexes(vec![0, 3]),
            false,
        )
        .expect("plan");
        SyntaxTree::parse(&result.rewritten).expect("rewritten parse");
        assert!(result.rewritten.starts_with("(defun render () (flet"));
    }
}

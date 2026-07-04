use anyhow::Result;

use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use crate::application::usecase::function_parameter::list_edit::{list_head, spans_overlap};

pub(in crate::application::usecase::function_parameter) fn resolve_function_call_paths(
    tree: &SyntaxTree,
    explicit_call_paths: Vec<Path>,
    all_calls: bool,
    definition_span: ByteSpan,
    function_name: &SymbolName,
    command: &str,
) -> Result<Vec<Path>> {
    if all_calls && !explicit_call_paths.is_empty() {
        anyhow::bail!("{command} accepts either --all-calls or repeated --call-path, not both");
    }

    if all_calls {
        let call_paths = discover_function_call_paths(tree, definition_span, function_name)?;
        if call_paths.is_empty() {
            anyhow::bail!(
                "{command} --all-calls found no same-file calls for {}",
                function_name
            );
        }
        return Ok(call_paths);
    }

    if explicit_call_paths.is_empty() {
        anyhow::bail!("{command} requires at least one --call-path or --all-calls");
    }

    Ok(explicit_call_paths)
}

fn discover_function_call_paths(
    tree: &SyntaxTree,
    definition_span: ByteSpan,
    function_name: &SymbolName,
) -> Result<Vec<Path>> {
    let mut call_paths = Vec::new();
    for index in 0..tree.root_children().len() {
        let mut indexes = vec![index];
        let path = Path::from_indexes(indexes.clone());
        let selection = tree.select_path(&path)?;
        let view = selection.view();
        collect_function_call_paths(
            &view,
            &mut indexes,
            definition_span,
            function_name,
            &mut call_paths,
        );
    }

    call_paths.sort_by_key(|path| {
        tree.select_path(path)
            .map(|selection| selection.span().start().get())
            .unwrap_or(usize::MAX)
    });
    Ok(call_paths)
}

fn collect_function_call_paths(
    view: &ExpressionView,
    indexes: &mut Vec<usize>,
    definition_span: ByteSpan,
    function_name: &SymbolName,
    output: &mut Vec<Path>,
) {
    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && !spans_overlap(definition_span, view.span)
        && list_head(view).is_some_and(|head| head == function_name.as_str())
    {
        output.push(Path::from_indexes(indexes.clone()));
    }

    for (index, child) in view.children.iter().enumerate() {
        indexes.push(index);
        collect_function_call_paths(child, indexes, definition_span, function_name, output);
        indexes.pop();
    }
}

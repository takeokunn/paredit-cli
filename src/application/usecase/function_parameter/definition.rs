use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::FunctionParameterInsert;
use super::list_edit::{atom_child, atom_text};

#[derive(Debug)]
pub(super) struct FunctionParameterTarget {
    pub(super) function_name: SymbolName,
    pub(super) parameter_container: ExpressionView,
    pub(super) protected_prefix_count: usize,
    pub(super) definition_span: ByteSpan,
    pub(super) has_lambda_list_marker: bool,
    pub(super) keyword_parameter_insertion: Option<KeywordParameterInsertion>,
    parameters: Vec<ParameterLocation>,
}

#[derive(Debug)]
pub(super) struct ParameterLocation {
    pub(super) name: String,
    pub(super) item_index: usize,
    pub(super) call_index: Option<usize>,
    pub(super) keyword_argument: Option<KeywordArgumentLocation>,
}

#[derive(Debug)]
pub(super) struct KeywordArgumentLocation {
    pub(super) keyword: String,
    pub(super) positional_prefix_count: usize,
}

#[derive(Debug)]
pub(super) struct KeywordParameterInsertion {
    pub(super) first_item_index: usize,
    pub(super) end_item_index: usize,
    pub(super) positional_prefix_count: usize,
    pub(super) keyword: String,
}

impl KeywordParameterInsertion {
    pub(super) fn item_index(&self, insert: FunctionParameterInsert) -> usize {
        match insert {
            FunctionParameterInsert::Start => self.first_item_index,
            FunctionParameterInsert::End => self.end_item_index,
        }
    }
}

struct LambdaListBinding<'a> {
    name: &'a str,
    keyword: Option<String>,
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

pub(super) fn parse_swap_function_parameters_definition(
    dialect: Dialect,
    view: ExpressionView,
) -> Result<FunctionParameterTarget> {
    parse_function_parameter_definition(dialect, view, None, "swap-function-parameters")
}

pub(super) fn parse_reorder_function_parameters_definition(
    dialect: Dialect,
    view: ExpressionView,
) -> Result<FunctionParameterTarget> {
    parse_function_parameter_definition(dialect, view, None, "reorder-function-parameters")
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

    let (
        function_name,
        parameter_container,
        protected_prefix_count,
        allow_specialized_required_parameters,
    ) = if head == "define" {
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
            false,
        )
    } else if common_lisp_method_head(head) {
        let name =
            atom_child(&view, 1).context("function definition must include a symbol name")?;
        let params = common_lisp_method_lambda_list(&view).with_context(|| {
            format!("{operation} defmethod definition must include a specialized lambda list")
        })?;
        (SymbolName::new(name.to_owned())?, params.clone(), 0, true)
    } else {
        let name =
            atom_child(&view, 1).context("function definition must include a symbol name")?;
        let params = view
            .children
            .get(2)
            .context("function definition must include a parameter list")?;
        (SymbolName::new(name.to_owned())?, params.clone(), 0, false)
    };
    let parameters = parameter_locations(
        dialect,
        &parameter_container,
        protected_prefix_count,
        allow_specialized_required_parameters,
        operation,
    )?;
    let has_lambda_list_marker = parameter_container.children[protected_prefix_count..]
        .iter()
        .any(|child| atom_text(child).is_some_and(|name| name.starts_with('&')));
    let keyword_parameter_insertion = match new_parameter {
        Some(new_parameter) => keyword_parameter_insertion(
            dialect,
            &parameter_container,
            protected_prefix_count,
            new_parameter,
        )?,
        None => None,
    };

    if let Some(new_parameter) = new_parameter {
        if new_parameter.as_str().starts_with(['&', ':']) {
            anyhow::bail!(
                "add-function-parameter found invalid parameter symbol '{}'",
                new_parameter
            );
        }
        if parameters
            .iter()
            .any(|parameter| parameter.name == new_parameter.as_str())
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
        has_lambda_list_marker,
        keyword_parameter_insertion,
        parameters,
    })
}

fn function_head_supported(dialect: Dialect, head: &str) -> bool {
    match dialect {
        Dialect::CommonLisp | Dialect::EmacsLisp => {
            matches!(head, "defun" | "defmacro") || common_lisp_method_head(head)
        }
        Dialect::Scheme => matches!(head, "define"),
        Dialect::Clojure => matches!(head, "defn" | "defmacro"),
        Dialect::Janet => matches!(head, "defn" | "defmacro"),
        Dialect::Fennel => matches!(head, "fn" | "lambda"),
        Dialect::Unknown => matches!(
            head,
            "defun"
                | "defmacro"
                | "define"
                | "defn"
                | "fn"
                | "lambda"
                | "defmethod"
                | "cl-defmethod"
        ),
    }
}

fn common_lisp_method_head(head: &str) -> bool {
    matches!(head, "defmethod" | "cl-defmethod")
}

fn common_lisp_method_lambda_list(view: &ExpressionView) -> Option<&ExpressionView> {
    view.children
        .iter()
        .skip(2)
        .find(|child| matches!(child.delimiter, Some(Delimiter::Paren | Delimiter::Bracket)))
}

fn parameter_locations(
    dialect: Dialect,
    parameter_form: &ExpressionView,
    protected_prefix_count: usize,
    allow_specialized_required_parameters: bool,
    operation: &str,
) -> Result<Vec<ParameterLocation>> {
    match parameter_form.kind {
        ExpressionKind::List => parameter_locations_from_children(
            dialect,
            &parameter_form.children,
            protected_prefix_count,
            allow_specialized_required_parameters,
            operation,
        ),
        _ => anyhow::bail!("{operation} function parameter form must be a list or vector"),
    }
}

fn parameter_locations_from_children(
    dialect: Dialect,
    children: &[ExpressionView],
    protected_prefix_count: usize,
    allow_specialized_required_parameters: bool,
    operation: &str,
) -> Result<Vec<ParameterLocation>> {
    let mut locations = Vec::with_capacity(children.len().saturating_sub(protected_prefix_count));
    let mut call_index = 0usize;
    let mut positional = true;
    let mut allow_lambda_list_spec = false;
    let mut keyword_parameters = false;
    let mut accepts_parameters = true;
    let supports_common_lisp_lambda_list = matches!(
        dialect,
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Unknown
    );

    for (item_index, child) in children.iter().enumerate().skip(protected_prefix_count) {
        if let Some(marker) = atom_text(child).filter(|name| name.starts_with('&')) {
            if !supports_common_lisp_lambda_list {
                anyhow::bail!(
                    "{operation} function parameter modifiers are not supported: {marker}"
                );
            }
            match marker {
                "&optional" => {
                    accepts_parameters = true;
                    positional = true;
                    allow_lambda_list_spec = true;
                    keyword_parameters = false;
                }
                "&key" => {
                    accepts_parameters = true;
                    positional = false;
                    allow_lambda_list_spec = true;
                    keyword_parameters = true;
                }
                "&aux" | "&rest" | "&body" | "&whole" | "&environment" => {
                    accepts_parameters = true;
                    positional = false;
                    allow_lambda_list_spec = marker == "&aux";
                    keyword_parameters = false;
                }
                "&allow-other-keys" => {
                    if !keyword_parameters {
                        anyhow::bail!(
                            "{operation} lambda-list marker &allow-other-keys is only supported after &key"
                        );
                    }
                    accepts_parameters = false;
                    positional = false;
                    allow_lambda_list_spec = false;
                    keyword_parameters = false;
                }
                _ => anyhow::bail!("{operation} unsupported lambda-list marker: {marker}"),
            }
            continue;
        }

        if !accepts_parameters {
            anyhow::bail!(
                "{operation} does not support parameters after &allow-other-keys before another lambda-list marker"
            );
        }
        let allow_specialized_required =
            allow_specialized_required_parameters && positional && !allow_lambda_list_spec;
        let binding = lambda_list_binding(
            child,
            allow_lambda_list_spec,
            keyword_parameters,
            allow_specialized_required,
        )
        .with_context(|| format!("{operation} currently supports only simple parameters"))?;
        SymbolName::new(binding.name.to_owned()).with_context(|| {
            format!(
                "{operation} found invalid parameter symbol '{}'",
                binding.name
            )
        })?;
        let call_index_for_parameter = positional.then_some(call_index);
        let keyword_argument = binding.keyword.map(|keyword| KeywordArgumentLocation {
            keyword,
            positional_prefix_count: call_index,
        });
        if positional {
            call_index += 1;
        }
        locations.push(ParameterLocation {
            name: binding.name.to_owned(),
            item_index,
            call_index: call_index_for_parameter,
            keyword_argument,
        });
    }
    Ok(locations)
}

fn keyword_parameter_insertion(
    dialect: Dialect,
    parameter_form: &ExpressionView,
    protected_prefix_count: usize,
    new_parameter: &SymbolName,
) -> Result<Option<KeywordParameterInsertion>> {
    if !matches!(
        dialect,
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Unknown
    ) {
        return Ok(None);
    }

    let mut positional_prefix_count = 0usize;
    let mut in_keyword_section = false;
    let mut first_item_index = None;
    let mut end_item_index = None;
    let mut positional_call_arguments = true;

    for (item_index, child) in parameter_form
        .children
        .iter()
        .enumerate()
        .skip(protected_prefix_count)
    {
        if let Some(marker) = atom_text(child).filter(|name| name.starts_with('&')) {
            match marker {
                "&key" => {
                    if first_item_index.is_some() {
                        anyhow::bail!("add-function-parameter found duplicate &key marker");
                    }
                    positional_call_arguments = false;
                    in_keyword_section = true;
                    first_item_index = Some(item_index + 1);
                }
                "&allow-other-keys" => {
                    positional_call_arguments = false;
                    if in_keyword_section && end_item_index.is_none() {
                        end_item_index = Some(item_index);
                    }
                    in_keyword_section = false;
                }
                "&optional" => {
                    positional_call_arguments = true;
                    if in_keyword_section && end_item_index.is_none() {
                        end_item_index = Some(item_index);
                    }
                    in_keyword_section = false;
                }
                _ => {
                    positional_call_arguments = false;
                    if in_keyword_section && end_item_index.is_none() {
                        end_item_index = Some(item_index);
                    }
                    in_keyword_section = false;
                }
            }
            continue;
        }

        if first_item_index.is_none() && positional_call_arguments {
            positional_prefix_count += 1;
        }
    }

    let Some(first_item_index) = first_item_index else {
        return Ok(None);
    };
    let end_item_index = end_item_index.unwrap_or(parameter_form.children.len());
    Ok(Some(KeywordParameterInsertion {
        first_item_index,
        end_item_index,
        positional_prefix_count,
        keyword: default_keyword_for_parameter(new_parameter.as_str()),
    }))
}

fn lambda_list_binding<'a>(
    child: &'a ExpressionView,
    allow_spec: bool,
    keyword_parameters: bool,
    allow_specialized_required: bool,
) -> Option<LambdaListBinding<'a>> {
    if let Some(name) = atom_text(child) {
        if keyword_parameters && name.starts_with(':') {
            return None;
        }
        return Some(LambdaListBinding {
            name,
            keyword: keyword_parameters.then(|| default_keyword_for_parameter(name)),
        });
    }
    if allow_specialized_required {
        if child.kind != ExpressionKind::List || child.children.len() != 2 {
            return None;
        }
        let name = atom_text(child.children.first()?)?;
        if name.starts_with('&') || name.starts_with(':') {
            return None;
        }
        return Some(LambdaListBinding {
            name,
            keyword: None,
        });
    }
    if !allow_spec {
        return None;
    }

    let binding = child.children.first()?;
    if let Some(name) = atom_text(binding) {
        if keyword_parameters && name.starts_with(':') {
            return None;
        }
        return Some(LambdaListBinding {
            name,
            keyword: keyword_parameters.then(|| default_keyword_for_parameter(name)),
        });
    }

    if keyword_parameters && binding.children.len() != 2 {
        return None;
    }
    let keyword = atom_text(binding.children.first()?)?;
    if keyword_parameters && !keyword.starts_with(':') {
        return None;
    }
    let name = binding.children.get(1).and_then(atom_text)?;
    Some(LambdaListBinding {
        name,
        keyword: keyword_parameters.then(|| keyword.to_owned()),
    })
}

fn default_keyword_for_parameter(name: &str) -> String {
    if name.starts_with(':') {
        name.to_owned()
    } else {
        format!(":{name}")
    }
}

pub(super) fn find_unique_parameter_location<'a>(
    target: &'a FunctionParameterTarget,
    parameter_name: &SymbolName,
    operation: &str,
) -> Result<&'a ParameterLocation> {
    let mut found = None;
    for parameter in &target.parameters {
        if parameter.name == parameter_name.as_str() && found.replace(parameter).is_some() {
            anyhow::bail!(
                "{operation} parameter '{}' appears more than once",
                parameter_name
            );
        }
    }

    found.with_context(|| format!("{operation} parameter '{}' was not found", parameter_name))
}

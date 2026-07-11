use anyhow::{Context, Result};

use crate::domain::common_lisp::common_lisp_symbol_name_eq;
use crate::domain::sexpr::SymbolName;

use super::types::{FunctionParameterTarget, ParameterLocation};

pub(crate) fn find_unique_parameter_location<'a>(
    target: &'a FunctionParameterTarget,
    parameter_name: &SymbolName,
    operation: &str,
) -> Result<&'a ParameterLocation> {
    let mut found = None;
    for parameter in &target.parameters {
        if common_lisp_symbol_name_eq(&parameter.name, parameter_name.as_str())
            && found.replace(parameter).is_some()
        {
            anyhow::bail!(
                "{operation} parameter '{}' appears more than once",
                parameter_name
            );
        }
    }

    found.with_context(|| format!("{operation} parameter '{}' was not found", parameter_name))
}

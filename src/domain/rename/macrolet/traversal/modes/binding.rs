use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::rename::macrolet::RenameFunctionOccurrence;
use crate::domain::sexpr::{ExpressionView, Path};

use super::super::core::RenameTraversalMode;
use super::super::state::{TraversalContext, TraversalState};
use super::common::{callable_binding_name_target, collect_active_atom_rename};

pub(in crate::domain::rename::macrolet) struct BindingTraversal;

impl RenameTraversalMode for BindingTraversal {
    fn collect_binding_name_renames(
        binding: &ExpressionView,
        binding_index: usize,
        path: &Path,
        form: crate::domain::common_lisp::CommonLispLocalCallableForm,
        context: TraversalContext<'_>,
        state: TraversalState,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) {
        if !context.kind.matches_target_form(form)
            || state.scope.is_target_active()
            || state.scope.is_shadowed()
        {
            return;
        }

        let Some(target) = callable_binding_name_target(binding, path, binding_index) else {
            return;
        };

        if !common_lisp_symbol_reference_eq(target.text, context.from.as_str()) {
            return;
        }

        renames.push(RenameFunctionOccurrence {
            path: target.path.to_string(),
            span: target.span,
            text: target.text.to_owned(),
            replacement: context.to.as_str().to_owned(),
        });
    }

    fn collect_explicit_function_lambda_atom_renames(
        child: &ExpressionView,
        child_path: &Path,
        context: TraversalContext<'_>,
        state: TraversalState,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        collect_active_atom_rename(child, child_path, context, state, state.scope, renames)
    }
}

use super::super::super::definition_removal::remove_unused_definitions::remove_unused_definitions_from_files;
use super::super::super::*;
use super::super::args::*;
use super::workspace::discover_workspace_refactor_scope;

pub(in crate::presentation::cli) fn workspace_remove_unused_definitions(
    args: WorkspaceRemoveUnusedDefinitionsArgs,
) -> Result<()> {
    let workspace = discover_workspace_refactor_scope(WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown,
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        exclude: args.exclude.clone(),
        max_depth: args.max_depth,
    })?;

    remove_unused_definitions_from_files(
        workspace.paths,
        None,
        args.include_protected,
        args.include_exported,
        args.write,
        args.output,
    )
}

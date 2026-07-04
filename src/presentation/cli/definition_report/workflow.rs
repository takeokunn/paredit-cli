use super::super::*;
use super::args::{DefinitionReportArgs, UnusedDefinitionReportArgs};
use super::render::{print_definition_report, print_unused_definition_report};
use crate::application::usecase::definition_report::{
    UnusedDefinitionPolicyOptions, build_definition_report, build_parsed_definition_file,
    collect_unused_definition_candidates, evaluate_unused_definition_policy,
};

pub(in crate::presentation::cli) fn definition_report(args: DefinitionReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        reports.push(build_definition_report(file.clone(), dialect, &tree)?);
    }

    print_definition_report(&reports, args.output)
}

pub(in crate::presentation::cli) fn unused_definition_report(
    args: UnusedDefinitionReportArgs,
) -> Result<()> {
    let mut parsed = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        parsed.push(build_parsed_definition_file(file.clone(), dialect, &tree)?);
    }

    let reports = collect_unused_definition_candidates(&parsed);
    let policy = evaluate_unused_definition_policy(
        UnusedDefinitionPolicyOptions {
            fail_on_unused: args.fail_on_unused,
            require_unused_definitions: args.require_unused_definitions,
        },
        &reports,
    );
    let policy_passed = policy.passed;
    let policy_message = policy.violations.join("; ");

    print_unused_definition_report(&reports, &policy, args.output)?;

    if !policy_passed {
        anyhow::bail!("unused-definition-report policy failed: {policy_message}");
    }

    Ok(())
}

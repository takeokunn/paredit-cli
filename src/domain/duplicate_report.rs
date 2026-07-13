//! Duplicate-form analysis and replacement planning rules.

use std::collections::BTreeMap;
use std::path::{Path as FsPath, PathBuf};

use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::form_shape::{FormShape, duplicate_shape};
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct DuplicateFormReport {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub form_path: Path,
    pub span: ByteSpan,
    pub node_count: usize,
    pub head: Option<String>,
    pub text: String,
}

#[derive(Debug)]
pub struct DuplicateShapeReport {
    pub shape: FormShape,
    pub count: usize,
    pub forms: Vec<DuplicateFormReport>,
}

#[derive(Debug)]
pub struct ReplacementPlanBatch {
    pub file: PathBuf,
    pub dialect: Dialect,
    pub shape: FormShape,
    pub replacement: String,
    pub keep_first: bool,
    pub forms: Vec<DuplicateFormReport>,
}

pub type DuplicateCandidateGroups = BTreeMap<FormShape, Vec<DuplicateFormReport>>;

pub fn collect_duplicate_candidates(
    tree: &SyntaxTree,
    input: &str,
    file: &FsPath,
    dialect: Dialect,
    min_node_count: usize,
    grouped: &mut DuplicateCandidateGroups,
) -> Result<()> {
    let mut path_stack = Vec::new();
    for index in 0..tree.root_children().len() {
        let view = tree.select_path(&Path::root_child(index))?.view();
        path_stack.push(index);
        collect_duplicate_candidates_from_view(
            &view,
            input,
            file,
            dialect,
            &mut path_stack,
            min_node_count,
            grouped,
        );
        path_stack.pop();
    }

    Ok(())
}

// `path_stack` is pushed/popped in place; a full `Path` is only built for
// forms that reach the candidate map, instead of cloning the whole index
// vector at every recursion step (O(nodes x depth) allocation).
fn collect_duplicate_candidates_from_view(
    view: &ExpressionView,
    input: &str,
    file: &FsPath,
    dialect: Dialect,
    path_stack: &mut Vec<usize>,
    min_node_count: usize,
    grouped: &mut DuplicateCandidateGroups,
) {
    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        let node_count = expression_node_count(view);
        if node_count >= min_node_count {
            let shape = duplicate_shape(view, true);
            grouped.entry(shape).or_default().push(DuplicateFormReport {
                path: file.to_path_buf(),
                dialect,
                form_path: Path::from_indexes(path_stack.clone()),
                span: view.span,
                node_count,
                head: view
                    .children
                    .first()
                    .and_then(atom_text)
                    .map(ToOwned::to_owned),
                text: view.span.slice(input).to_owned(),
            });
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        path_stack.push(index);
        collect_duplicate_candidates_from_view(
            child,
            input,
            file,
            dialect,
            path_stack,
            min_node_count,
            grouped,
        );
        path_stack.pop();
    }
}

pub fn collect_replacement_plan_batches(
    grouped: DuplicateCandidateGroups,
    min_group_size: usize,
    replacement: String,
    keep_first: bool,
) -> Vec<ReplacementPlanBatch> {
    let mut batches = Vec::new();

    for (shape, forms) in grouped {
        let mut by_file = BTreeMap::<PathBuf, Vec<DuplicateFormReport>>::new();
        for form in forms {
            by_file.entry(form.path.clone()).or_default().push(form);
        }

        for (file, mut file_forms) in by_file {
            if file_forms.len() < min_group_size {
                continue;
            }

            file_forms.sort_by_key(|form| form.span.start().get());
            let Some(first_form) = file_forms.first() else {
                continue;
            };

            batches.push(ReplacementPlanBatch {
                file,
                dialect: first_form.dialect,
                shape: shape.clone(),
                replacement: replacement.clone(),
                keep_first,
                forms: file_forms,
            });
        }
    }

    batches
}

pub fn build_duplicate_shape_reports(
    grouped: DuplicateCandidateGroups,
    min_group_size: usize,
) -> Vec<DuplicateShapeReport> {
    let mut reports = grouped
        .into_iter()
        .filter_map(|(shape, forms)| {
            (forms.len() >= min_group_size).then_some(DuplicateShapeReport {
                count: forms.len(),
                shape,
                forms,
            })
        })
        .collect::<Vec<_>>();

    reports.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.shape.cmp(&right.shape))
    });

    reports
}

fn expression_node_count(view: &ExpressionView) -> usize {
    1 + view
        .children
        .iter()
        .map(expression_node_count)
        .sum::<usize>()
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .and_then(|text| text)
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use std::path::{Path as FsPath, PathBuf};

    use proptest::prelude::*;

    use super::*;
    use crate::domain::sexpr::{ByteOffset, Path as ExpressionPath};

    #[test]
    fn groups_duplicate_forms_by_shape() {
        let input = "(+ a b)\n(+ c d)\n(* a b)\n";
        let tree = SyntaxTree::parse(input).expect("parse input");
        let mut grouped = DuplicateCandidateGroups::new();

        collect_duplicate_candidates(
            &tree,
            input,
            FsPath::new("sample.lisp"),
            Dialect::CommonLisp,
            3,
            &mut grouped,
        )
        .expect("collect candidates");

        let reports = build_duplicate_shape_reports(grouped, 2);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].count, 2);
        assert_eq!(
            reports[0].forms[0].form_path,
            ExpressionPath::from_indexes(vec![0])
        );
        assert_eq!(
            reports[0].forms[1].form_path,
            ExpressionPath::from_indexes(vec![1])
        );
    }

    #[test]
    fn replacement_batches_are_partitioned_per_file() {
        let shape = FormShape::from("(+ _ _)");
        let span = ByteSpan::new(ByteOffset::new(0), ByteOffset::new(7));
        let mut grouped = DuplicateCandidateGroups::new();
        grouped.insert(
            shape.clone(),
            vec![
                DuplicateFormReport {
                    path: PathBuf::from("a.lisp"),
                    dialect: Dialect::CommonLisp,
                    form_path: ExpressionPath::from_indexes(vec![1]),
                    span,
                    node_count: 4,
                    head: Some("+".to_owned()),
                    text: "(+ c d)".to_owned(),
                },
                DuplicateFormReport {
                    path: PathBuf::from("a.lisp"),
                    dialect: Dialect::CommonLisp,
                    form_path: ExpressionPath::from_indexes(vec![0]),
                    span: ByteSpan::new(ByteOffset::new(8), ByteOffset::new(15)),
                    node_count: 4,
                    head: Some("+".to_owned()),
                    text: "(+ a b)".to_owned(),
                },
                DuplicateFormReport {
                    path: PathBuf::from("b.lisp"),
                    dialect: Dialect::CommonLisp,
                    form_path: ExpressionPath::from_indexes(vec![0]),
                    span,
                    node_count: 4,
                    head: Some("+".to_owned()),
                    text: "(+ e f)".to_owned(),
                },
            ],
        );

        let batches =
            collect_replacement_plan_batches(grouped, 2, "(helper _ _)".to_owned(), false);

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].file, PathBuf::from("a.lisp"));
        assert_eq!(batches[0].shape, shape);
        assert!(!batches[0].keep_first);
        assert_eq!(batches[0].forms[0].span.start().get(), 0);
        assert_eq!(batches[0].forms[1].span.start().get(), 8);
    }

    proptest! {
        #[test]
        fn pbt_repeated_binary_calls_are_reported_as_one_duplicate_shape(
            count in 2usize..12,
            head in "[a-z]{1,8}",
            lhs in "[a-z]{1,8}",
            rhs in "[a-z]{1,8}",
        ) {
            let forms = (0..count)
                .map(|index| format!("({head} {lhs}{index} {rhs}{index})"))
                .collect::<Vec<_>>();
            let input = forms.join("\n");
            let tree = SyntaxTree::parse(&input).expect("parse generated input");
            let mut grouped = DuplicateCandidateGroups::new();

            collect_duplicate_candidates(
                &tree,
                &input,
                FsPath::new("generated.lisp"),
                Dialect::CommonLisp,
                3,
                &mut grouped,
            )
            .expect("collect generated candidates");

            let reports = build_duplicate_shape_reports(grouped, count);
            prop_assert_eq!(reports.len(), 1);
            prop_assert_eq!(reports[0].count, count);
            prop_assert!(reports[0]
                .forms
                .iter()
                .all(|form| form.head.as_deref() == Some(head.as_str())));
        }
    }
}

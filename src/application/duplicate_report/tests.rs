use std::path::{Path, PathBuf};

use proptest::prelude::*;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteOffset, ByteSpan, SyntaxTree};

use super::*;

#[test]
fn groups_duplicate_forms_by_shape() {
    let input = "(+ a b)\n(+ c d)\n(* a b)\n";
    let tree = SyntaxTree::parse(input).expect("parse input");
    let mut grouped = DuplicateCandidateGroups::new();

    collect_duplicate_candidates(
        &tree,
        input,
        Path::new("sample.lisp"),
        Dialect::CommonLisp,
        3,
        &mut grouped,
    )
    .expect("collect candidates");

    let reports = build_duplicate_shape_reports(grouped, 2);
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].count, 2);
    assert_eq!(reports[0].forms[0].form_path, "0");
    assert_eq!(reports[0].forms[1].form_path, "1");
}

#[test]
fn replacement_batches_are_partitioned_per_file() {
    let shape = "(+ _ _)".to_owned();
    let span = ByteSpan::new(ByteOffset::new(0), ByteOffset::new(7));
    let mut grouped = DuplicateCandidateGroups::new();
    grouped.insert(
        shape.clone(),
        vec![
            DuplicateFormReport {
                path: PathBuf::from("a.lisp"),
                dialect: Dialect::CommonLisp,
                form_path: "1".to_owned(),
                span,
                node_count: 4,
                head: Some("+".to_owned()),
                text: "(+ c d)".to_owned(),
            },
            DuplicateFormReport {
                path: PathBuf::from("a.lisp"),
                dialect: Dialect::CommonLisp,
                form_path: "0".to_owned(),
                span: ByteSpan::new(ByteOffset::new(8), ByteOffset::new(15)),
                node_count: 4,
                head: Some("+".to_owned()),
                text: "(+ a b)".to_owned(),
            },
            DuplicateFormReport {
                path: PathBuf::from("b.lisp"),
                dialect: Dialect::CommonLisp,
                form_path: "0".to_owned(),
                span,
                node_count: 4,
                head: Some("+".to_owned()),
                text: "(+ e f)".to_owned(),
            },
        ],
    );

    let batches = collect_replacement_plan_batches(grouped, 2, "(helper _ _)".to_owned());

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].file, PathBuf::from("a.lisp"));
    assert_eq!(batches[0].shape, shape);
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
            Path::new("generated.lisp"),
            Dialect::CommonLisp,
            3,
            &mut grouped,
        )
        .expect("collect generated candidates");

        let reports = build_duplicate_shape_reports(grouped, count);
        prop_assert_eq!(reports.len(), 1);
        prop_assert_eq!(reports[0].count, count);
        prop_assert!(reports[0].forms.iter().all(|form| form.head.as_deref() == Some(head.as_str())));
    }
}

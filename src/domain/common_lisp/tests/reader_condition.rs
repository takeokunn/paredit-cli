use super::*;
use crate::domain::common_lisp::{
    common_lisp_reader_conditional_dispatches, contains_common_lisp_reader_conditional,
    CommonLispReaderConditionalKind,
};

#[test]
fn detects_bare_include_and_exclude_dispatches() {
    let tree = SyntaxTree::parse("#+ #-").expect("parse succeeds");

    let dispatches = common_lisp_reader_conditional_dispatches(&tree);

    assert_eq!(
        dispatches
            .iter()
            .map(|dispatch| dispatch.kind)
            .collect::<Vec<_>>(),
        vec![
            CommonLispReaderConditionalKind::Include,
            CommonLispReaderConditionalKind::Exclude,
        ]
    );
    assert_eq!(
        dispatches
            .iter()
            .map(|dispatch| dispatch.path.to_string())
            .collect::<Vec<_>>(),
        vec!["0", "1"]
    );
    assert!(contains_common_lisp_reader_conditional(&tree));
}

#[test]
fn detects_simple_and_compound_feature_conditions() {
    let tree = SyntaxTree::parse("#+sbcl (compile-file source) #-(and sbcl x86-64) (load source)")
        .expect("parse succeeds");

    let dispatches = common_lisp_reader_conditional_dispatches(&tree);

    assert_eq!(dispatches.len(), 2);
    assert_eq!(dispatches[0].kind, CommonLispReaderConditionalKind::Include);
    assert_eq!(dispatches[1].kind, CommonLispReaderConditionalKind::Exclude);
    assert_eq!(
        dispatches[0]
            .span
            .slice("#+sbcl (compile-file source) #-(and sbcl x86-64) (load source)"),
        "#+"
    );
    assert_eq!(
        dispatches[1]
            .span
            .slice("#+sbcl (compile-file source) #-(and sbcl x86-64) (load source)"),
        "#-"
    );
}

#[test]
fn detects_dispatches_with_quote_and_function_reader_prefixes() {
    let tree = SyntaxTree::parse("'#+sbcl selected #'#-sbcl rejected").expect("parse succeeds");

    let dispatches = common_lisp_reader_conditional_dispatches(&tree);

    assert_eq!(dispatches.len(), 2);
    assert_eq!(dispatches[0].kind, CommonLispReaderConditionalKind::Include);
    assert_eq!(dispatches[1].kind, CommonLispReaderConditionalKind::Exclude);
    assert_eq!(
        dispatches[0]
            .span
            .slice("'#+sbcl selected #'#-sbcl rejected"),
        "#+"
    );
    assert_eq!(
        dispatches[1]
            .span
            .slice("'#+sbcl selected #'#-sbcl rejected"),
        "#-"
    );
}

#[test]
fn detects_multiple_and_nested_dispatches() {
    let tree = SyntaxTree::parse("(progn #+outer (progn #-inner skipped) #-fallback (run))")
        .expect("parse succeeds");

    let dispatches = common_lisp_reader_conditional_dispatches(&tree);

    assert_eq!(
        dispatches
            .iter()
            .map(|dispatch| dispatch.kind)
            .collect::<Vec<_>>(),
        vec![
            CommonLispReaderConditionalKind::Include,
            CommonLispReaderConditionalKind::Exclude,
            CommonLispReaderConditionalKind::Exclude,
        ]
    );
    assert_eq!(
        dispatches
            .iter()
            .map(|dispatch| dispatch.path.to_string())
            .collect::<Vec<_>>(),
        vec!["0.1", "0.3.1", "0.4"]
    );
}

#[test]
fn does_not_confuse_clojure_conditionals_or_reader_comments_with_common_lisp_dispatches() {
    for input in ["#?(:clj selected :cljs ignored)", "#; #+sbcl ignored"] {
        let tree = SyntaxTree::parse(input).expect("parse succeeds");

        assert!(common_lisp_reader_conditional_dispatches(&tree).is_empty());
        assert!(!contains_common_lisp_reader_conditional(&tree));
    }
}

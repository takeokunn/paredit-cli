use super::*;

#[test]
fn cli_filters_pairs_by_comparison_scope() {
    let dir = fresh_temp_dir("similarity-comparison-scope");
    let left = dir.join("left.lisp");
    let right = dir.join("right.lisp");
    fs::write(&left, "(foo a b) (foo a b)\n").unwrap();
    fs::write(&right, "(foo a b) (foo a b)\n").unwrap();

    for (scope, same_file) in [("same-file", true), ("cross-file", false)] {
        let output = paredit()
            .args(["inspect", "similarity"])
            .arg("--threshold=1")
            .arg("--min-node-count=2")
            .arg("--overlap-policy=all")
            .arg(format!("--comparison-scope={scope}"))
            .arg(&left)
            .arg(&right)
            .output()
            .unwrap();
        assert!(output.status.success());
        let report = parse_similarity_report(&output.stdout);
        assert_eq!(report["options"]["comparison_scope"], scope);
        let pairs = report["pairs"].as_array().unwrap();
        assert!(!pairs.is_empty());
        assert!(
            pairs
                .iter()
                .all(|pair| { (pair["left"]["path"] == pair["right"]["path"]) == same_file })
        );
    }
}

#[test]
fn cli_top_level_form_scope_excludes_nested_candidates() {
    let dir = fresh_temp_dir("similarity-form-scope");
    let file = dir.join("nested.lisp");
    fs::write(&file, "(outer (foo a b) (foo a b))\n").unwrap();

    let all_output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--min-node-count=2")
        .arg("--overlap-policy=all")
        .arg("--form-scope=all")
        .arg(&file)
        .output()
        .unwrap();
    let top_level_output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--min-node-count=2")
        .arg("--overlap-policy=all")
        .arg("--form-scope=top-level")
        .arg(&file)
        .output()
        .unwrap();

    assert!(all_output.status.success());
    assert!(top_level_output.status.success());
    let all: Value = serde_json::from_slice(&all_output.stdout).unwrap();
    let top_level: Value = serde_json::from_slice(&top_level_output.stdout).unwrap();
    assert!(all["pair_count"].as_u64().unwrap() > 0);
    assert_eq!(top_level["pair_count"], 0);
    assert_eq!(top_level["options"]["form_scope"], "top-level");
}

#[test]
fn cli_min_line_span_keeps_only_multiline_candidates() {
    let dir = fresh_temp_dir("similarity-min-line-span");
    let file = dir.join("lines.lisp");
    fs::write(&file, "(foo a b) (foo a b)\n(foo\n a\n b)\n(foo\n a\n b)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--min-node-count=2")
        .arg("--min-line-span=2")
        .arg("--overlap-policy=all")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["options"]["min_line_span"], 2);
    assert_eq!(report["pair_count"], 1);
    assert!(report["pairs"].as_array().unwrap().iter().all(|pair| {
        pair["left"]["text"].as_str().unwrap().contains('\n')
            && pair["right"]["text"].as_str().unwrap().contains('\n')
    }));
}

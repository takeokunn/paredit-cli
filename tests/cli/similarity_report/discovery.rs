use super::*;

#[test]
fn cli_recursively_discovers_sources_and_skips_hidden_and_generated_directories() {
    let dir = fresh_temp_dir("similarity-discovery");
    let nested = dir.join("src").join("nested");
    let hidden = dir.join(".private");
    let generated = dir.join("target");
    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(&hidden).unwrap();
    fs::create_dir_all(&generated).unwrap();
    fs::write(nested.join("left.lisp"), "(foo a b)\n").unwrap();
    fs::write(dir.join("right.lisp"), "(foo x y)\n").unwrap();
    fs::write(hidden.join("hidden.lisp"), "(foo h i)\n").unwrap();
    fs::write(generated.join("generated.lisp"), "(foo g h)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold")
        .arg("0")
        .arg("--min-node-count")
        .arg("2")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["summary"]["scanned_files"], 2);
    assert_eq!(report["summary"]["skipped_hidden"], 1);
    assert_eq!(report["summary"]["skipped_generated"], 1);
    assert_eq!(report["summary"]["matched_pairs"], 1);
}

#[test]
fn cli_excludes_repeated_file_and_subtree_paths_without_prefix_false_positives() {
    let dir = fresh_temp_dir("similarity-exclude-json");
    let excluded_dir = dir.join("vendor");
    let prefix_sibling = dir.join("vendor-copy");
    let excluded_file = dir.join("excluded.lisp");
    fs::create_dir_all(&excluded_dir).unwrap();
    fs::create_dir_all(&prefix_sibling).unwrap();
    fs::write(dir.join("keep.lisp"), "(foo a b)\n").unwrap();
    fs::write(&excluded_file, "(foo excluded file)\n").unwrap();
    fs::write(excluded_dir.join("nested.lisp"), "(foo excluded dir)\n").unwrap();
    fs::write(prefix_sibling.join("keep.lisp"), "(foo x y)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=0")
        .arg("--exclude")
        .arg(&excluded_dir)
        .arg("--exclude")
        .arg(&excluded_file)
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(
        report["options"]["exclude"],
        serde_json::json!([
            excluded_dir.display().to_string(),
            excluded_file.display().to_string()
        ])
    );
    assert_eq!(report["summary"]["scanned_files"], 2);
    assert_eq!(report["summary"]["skipped_excluded"], 2);
    assert_eq!(report["pair_count"], 1);
}

#[test]
fn cli_resolves_relative_excludes_from_cwd_and_reports_text_summary() {
    let dir = fresh_temp_dir("similarity-exclude-text");
    fs::write(dir.join("keep.lisp"), "(foo a b)\n").unwrap();
    fs::write(dir.join("excluded.lisp"), "(foo x y)\n").unwrap();

    paredit()
        .current_dir(&dir)
        .args(["inspect", "similarity"])
        .arg("--output=text")
        .arg("--exclude=excluded.lisp")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("scanned_files\t1"))
        .stdout(predicate::str::contains("skipped_excluded\t1"));
}

#[test]
fn cli_dialect_override_includes_unknown_extensions() {
    let dir = fresh_temp_dir("similarity-dialect-override");
    let left = dir.join("left.txt");
    let right = dir.join("right.data");
    fs::write(&left, "(foo a b)\n").unwrap();
    fs::write(&right, "(foo x y)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--dialect")
        .arg("common-lisp")
        .arg("--threshold")
        .arg("0")
        .arg("--min-node-count")
        .arg("2")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["options"]["dialect"], "common-lisp");
    assert_eq!(report["summary"]["scanned_files"], 2);
    assert_eq!(report["summary"]["skipped_unknown"], 0);
    assert!(report["pairs"].as_array().unwrap().iter().all(|pair| {
        pair["left"]["dialect"] == "common-lisp" && pair["right"]["dialect"] == "common-lisp"
    }));
}

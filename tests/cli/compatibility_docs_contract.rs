#[test]
fn compatibility_policy_declares_no_backward_compatibility() {
    let compatibility = std::fs::read_to_string("COMPATIBILITY.md").expect("read COMPATIBILITY");

    for needle in [
        "does not provide backward compatibility",
        "`paredit inspect ...`",
        "`paredit edit ...`",
        "`paredit refactor ...`",
    ] {
        assert!(
            compatibility.contains(needle),
            "COMPATIBILITY must keep the Lisp scope boundary explicit: {needle}"
        );
    }
}

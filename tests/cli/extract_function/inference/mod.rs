use super::*;

mod basic;
mod bindings;
mod local_callables;
mod macro_lambda_lists;

fn json_array_field(name: &str, values: &[&str]) -> String {
    let body = values
        .iter()
        .map(|value| format!("    \"{value}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("\"{name}\": [\n{body}\n  ]")
}

fn assert_extract_function_inference(
    args: &[&str],
    input: &str,
    params: &[&str],
    inferred_params: &[&str],
    call: &str,
    definition: Option<&str>,
) {
    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("extract-function")
        .args(args)
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains(json_array_field("params", params)))
        .stdout(predicate::str::contains(json_array_field(
            "inferred_params",
            inferred_params,
        )))
        .stdout(predicate::str::contains(format!("\"call\": \"{call}\"")));

    if let Some(definition) = definition {
        cmd.assert().stdout(predicate::str::contains(format!(
            "\"definition\": \"{definition}\""
        )));
    }
}

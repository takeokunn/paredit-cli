use super::*;

mod basics;
mod policy;
mod shadowing;

fn write_call_graph_fixture(dir_name: &str, file_name: &str, source: &str) -> PathBuf {
    let dir = fresh_temp_dir(dir_name);
    let path = dir.join(file_name);
    fs::write(&path, source).expect("write call-graph fixture");
    path
}

fn assert_shadowed_helper_edges(stdout: &str) {
    assert!(stdout.contains("\"edge_count\": 2"));
    assert!(stdout.contains("\"path\": \"1.3.1.0.2\""));
    assert!(stdout.contains("\"path\": \"1.4\""));
    assert!(stdout.contains("\"callee\": \"helper\""));
    assert!(stdout.contains("\"caller\": \"render\""));
}

#![allow(
    clippy::expect_used,
    reason = "tests use expect to keep fixture setup concise"
)]

use crate::common::{parse_json, run_fallow_in_root};
use std::path::Path;
use tempfile::tempdir;

const NESTED_FN: &str = "function nested(a, b) { if (a) { if (b) { return 1; } } return 0; }";

fn reported_functions(root: &Path, page: &str) -> Vec<String> {
    std::fs::write(root.join("src/pages/index.astro"), page).expect("write page");
    let output = run_fallow_in_root(
        "health",
        root,
        &[
            "--complexity",
            "--max-cyclomatic",
            "1",
            "--max-cognitive",
            "1",
            "--max-crap",
            "10000",
            "--report-only",
            "--format",
            "json",
            "--quiet",
            "--no-cache",
        ],
    );
    assert_eq!(output.code, 0, "stderr: {}", output.stderr);
    parse_json(&output)["findings"]
        .as_array()
        .expect("findings array")
        .iter()
        .filter_map(|finding| finding["name"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn health_astro_script_suppression_comments_hide_function_findings() {
    let dir = tempdir().expect("create temp dir");
    std::fs::create_dir_all(dir.path().join("src/pages")).expect("create pages dir");
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"name":"astro-script-suppression","dependencies":{"astro":"latest"}}"#,
    )
    .expect("write package.json");

    let control = reported_functions(dir.path(), &format!("<script>\n{NESTED_FN}\n</script>\n"));
    assert_eq!(control, vec!["nested"]);

    let line_comment = reported_functions(
        dir.path(),
        &format!("<script>\n// fallow-ignore-next-line complexity\n{NESTED_FN}\n</script>\n"),
    );
    assert!(line_comment.is_empty(), "{line_comment:?}");

    let html_comment = reported_functions(
        dir.path(),
        &format!("<!-- fallow-ignore-next-line complexity -->\n<script>{NESTED_FN}</script>\n"),
    );
    assert!(html_comment.is_empty(), "{html_comment:?}");
}

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;

#[test]
fn generate_page_creates_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("pages")).expect("mkdir pages");

    let mut cmd = Command::cargo_bin("vega-cli").expect("bin");
    cmd.args([
        "generate",
        "page",
        "about",
        "--path",
        temp.path().to_str().expect("path"),
    ]);
    cmd.assert().success();

    assert!(temp.path().join("pages/about.rs").exists());
}

#[test]
fn routes_prints_discovered_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("pages")).expect("mkdir pages");
    fs::create_dir_all(temp.path().join("api")).expect("mkdir api");
    fs::write(temp.path().join("pages/about.rs"), "pub fn About() {}\n").expect("write page");
    fs::write(
        temp.path().join("api/hello.rs"),
        "#[get]\npub fn handler() {}\n",
    )
    .expect("write api");

    let mut cmd = Command::cargo_bin("vega-cli").expect("bin");
    cmd.args(["routes", "--path", temp.path().to_str().expect("path")]);
    cmd.assert()
        .success()
        .stdout(contains("/about"))
        .stdout(contains("/api/hello"));
}

use std::path::Path;
use std::process::Command;

fn main() {
    let output = Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .expect("Failed to execute `cargo locate-project`");

    if !output.status.success() {
        panic!(
            "cargo locate-project failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let cargo_toml_path = Path::new(std::str::from_utf8(&output.stdout).unwrap().trim());
    let workspace_root = cargo_toml_path
        .parent()
        .expect("Failed to find workspace root from Cargo.toml path");

    println!(
        "cargo:rustc-env=WORKSPACE_ROOT={}",
        workspace_root.display()
    );

    println!("cargo:rerun-if-changed={}", cargo_toml_path.display());
}

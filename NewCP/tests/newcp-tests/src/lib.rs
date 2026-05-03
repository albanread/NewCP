#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn workspace_root() -> PathBuf {
        // CARGO_MANIFEST_DIR = NewCP/tests/newcp-tests
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent() // tests/
            .unwrap()
            .parent() // NewCP/
            .unwrap()
            .to_path_buf()
    }

    fn driver_bin() -> PathBuf {
        let bin = workspace_root()
            .join("target")
            .join("debug")
            .join(if cfg!(windows) { "newcp-driver.exe" } else { "newcp-driver" });
        // If the binary doesn't exist yet, build it now so tests can run standalone.
        if !bin.exists() {
            let status = Command::new("cargo")
                .args(["build", "-p", "newcp-driver"])
                .current_dir(workspace_root())
                .status()
                .expect("failed to run cargo build for newcp-driver");
            assert!(status.success(), "cargo build -p newcp-driver failed");
        }
        bin
    }

    /// Run `check-mod <module>` from the workspace root and return (stdout, exit_code).
    fn check_mod(module: &str) -> (String, i32) {
        let out = Command::new(driver_bin())
            .args(["check-mod", module])
            .current_dir(workspace_root())
            .output()
            .expect("failed to spawn driver binary");

        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let code = out.status.code().unwrap_or(-1);
        (stdout, code)
    }

    #[test]
    fn check_mod_empty_is_clean() {
        let (output, code) = check_mod("Empty");
        assert_eq!(code, 0, "expected exit 0 for Empty.cp\noutput:\n{output}");
        assert!(
            output.trim_end().ends_with("ok"),
            "expected 'ok' in output for Empty.cp\noutput:\n{output}"
        );
    }
}

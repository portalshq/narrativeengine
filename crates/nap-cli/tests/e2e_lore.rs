/// End-to-end tests that exercise the full CLI → Lore daemon → Repository flow.
///
/// These require:
/// 1. A running `lore` daemon (`NAP_LORE_URL_BASE` must be set)
/// 2. The `lore` binary in PATH
///
/// Run with:
///   cargo test -p nap-cli --features lore-e2e -- --test-threads=1

#[cfg(feature = "lore-e2e")]
mod e2e {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::TempDir;

    fn nap_cmd() -> Command {
        let mut cmd = Command::cargo_bin("nap").expect("Failed to find nap binary");
        cmd.timeout(std::time::Duration::from_secs(30));
        cmd
    }

    #[test]
    fn test_e2e_init_and_resolve() {
        let tmp = TempDir::new().unwrap();
        let repository = "e2e-test-repository";

        nap_cmd()
            .arg("init")
            .arg(repository)
            .arg("--base-dir")
            .arg(tmp.path())
            .assert()
            .success();

        nap_cmd()
            .arg("resolve")
            .arg(format!("{repository}/character/nonexistent"))
            .arg("--base-dir")
            .arg(tmp.path())
            .assert()
            .failure();
    }

    #[test]
    fn test_e2e_create_entity_and_resolve() {
        let tmp = TempDir::new().unwrap();
        let repository = "e2e-create-resolve";

        nap_cmd()
            .arg("init")
            .arg(repository)
            .arg("--base-dir")
            .arg(tmp.path())
            .assert()
            .success();

        nap_cmd()
            .arg("create")
            .arg("--base-dir")
            .arg(tmp.path())
            .arg("--repository")
            .arg(repository)
            .arg("--type")
            .arg("character")
            .arg("--id")
            .arg("hero")
            .arg("--name")
            .arg("E2E Hero")
            .assert()
            .success();

        nap_cmd()
            .arg("resolve")
            .arg(format!("{repository}/character/hero"))
            .arg("--base-dir")
            .arg(tmp.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("E2E Hero"));
    }
}

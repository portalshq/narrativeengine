use crate::repository::Repository;
use crate::test_utils::mock_backend::MockBackend;
use tempfile::TempDir;

pub fn mock_repo(tmp: &TempDir) -> Repository {
    let repo_path = tmp.path().join("testverse");
    Repository::init(&repo_path, "testverse", Box::new(MockBackend::new())).unwrap()
}

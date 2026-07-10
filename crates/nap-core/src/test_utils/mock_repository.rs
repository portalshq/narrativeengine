use crate::repository::Repository;
use crate::test_utils::mock_backend::MockBackend;
use tempfile::TempDir;

pub fn mock_repo(tmp: &TempDir) -> Repository {
    Repository::init(tmp.path(), "testverse", Box::new(MockBackend::new())).unwrap()
}

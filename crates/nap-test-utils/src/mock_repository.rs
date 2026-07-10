use crate::mock_backend::MockBackend;
use nap_core::repository::Repository;
use tempfile::TempDir;

pub fn mock_repo(tmp: &TempDir) -> Repository {
    Repository::init(tmp.path(), "testverse", Box::new(MockBackend::new())).unwrap()
}

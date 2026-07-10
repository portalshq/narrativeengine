use nap_core::commit::Change;
use nap_core::repository::Repository;
use nap_core::types::EntityType;
use nap_core::vcs::VcsBackend;
use tempfile::TempDir;

pub fn run_repository_contract(backend: impl VcsBackend + 'static) {
    let tmp = TempDir::new().unwrap();
    let repo = Repository::init(tmp.path(), "contract-test", Box::new(backend)).unwrap();

    // Contract: init creates structure
    assert!(repo.root.join(".nap").exists());
    assert!(repo.root.join("universe.yaml").exists());

    // Contract: create and read entity
    let (manifest, _) = repo
        .create_entity(EntityType::Character, "hero", "Test Hero", "contract")
        .unwrap();
    assert_eq!(manifest.name, "Test Hero");

    let read_back = repo.read_manifest(EntityType::Character, "hero").unwrap();
    assert_eq!(read_back.name, "Test Hero");

    // Contract: commit updates version
    let mut m = read_back;
    m.set_property("test", serde_yaml::Value::Bool(true));
    repo.commit_manifest(
        &mut m,
        "contract commit",
        "contract",
        vec![Change::set("properties.test", None, "true".to_string())],
    )
    .unwrap();
    let after = repo.read_manifest(EntityType::Character, "hero").unwrap();
    assert!(after.version >= 2);

    // Contract: list entities
    let entities = repo.list_entities(EntityType::Character).unwrap();
    assert!(entities.contains(&"hero".to_string()));

    // Contract: history is non-empty
    let hist = repo.history(EntityType::Character, "hero", 10).unwrap();
    assert!(!hist.is_empty());

    // Contract: branch operations
    repo.create_branch("feature").unwrap();
    let branches = repo.list_branches().unwrap();
    assert!(branches.contains(&"feature".to_string()));
}

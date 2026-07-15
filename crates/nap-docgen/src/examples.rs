use std::fs;
use std::path::Path;

pub fn load_example_snippet(workspace_root: &Path, command_name: &str) -> Option<String> {
    let path = workspace_root
        .join("docs")
        .join("examples")
        .join(format!("{command_name}.md"));

    if path.exists() {
        fs::read_to_string(&path).ok()
    } else {
        None
    }
}

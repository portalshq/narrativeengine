use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

struct CycleDetector {
    stack: HashSet<PathBuf>,
}

impl CycleDetector {
    fn new() -> Self {
        Self {
            stack: HashSet::new(),
        }
    }

    fn enter(&mut self, path: PathBuf) -> Result<()> {
        if self.stack.contains(&path) {
            bail!(
                "Circular include detected: {} is already being processed",
                path.display()
            );
        }
        self.stack.insert(path);
        Ok(())
    }

    fn exit(&mut self, path: &Path) {
        self.stack.remove(path);
    }
}

pub fn expand_template(
    content: &str,
    workspace_root: &Path,
    variables: &[(String, String)],
) -> Result<String> {
    let mut detector = CycleDetector::new();
    expand_recursive(content, workspace_root, variables, &mut detector)
}

fn expand_recursive(
    content: &str,
    workspace_root: &Path,
    variables: &[(String, String)],
    detector: &mut CycleDetector,
) -> Result<String> {
    let mut result = content.to_string();

    // Expand includes until no more remain
    while let Some(include_pos) = result.find("{{include ") {
        let start = include_pos;
        let end = result[start..]
            .find("}}")
            .context("unclosed {{include directive")?;
        let directive = &result[start..start + end + 2];
        let path_str = directive
            .strip_prefix("{{include ")
            .and_then(|s| s.strip_suffix("}}"))
            .context("malformed include directive")?
            .trim();

        let include_path = workspace_root.join(path_str);

        if !include_path.exists() {
            bail!(
                "Include target not found: {} (resolved from {})",
                include_path.display(),
                path_str
            );
        }

        let canonical = include_path
            .canonicalize()
            .context("failed to canonicalize include path")?;

        detector.enter(canonical.clone())?;

        let included_content =
            fs::read_to_string(&include_path).context("failed to read include file")?;

        // Strip YAML front matter from generated docs so they render cleanly when included
        let included_content = strip_front_matter(&included_content);

        let expanded = expand_recursive(&included_content, workspace_root, variables, detector)?;

        detector.exit(&canonical);

        result = format!(
            "{}{}{}",
            &result[..start],
            expanded,
            &result[start + end + 2..]
        );
    }

    // Expand variables
    for (key, value) in variables {
        let placeholder = format!("{{{{{key}}}}}");
        result = result.replace(&placeholder, value);
    }

    Ok(result)
}

/// Strip YAML front matter (---\n...\n) from content if present.
fn strip_front_matter(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
        return content.to_string();
    }
    // Find the closing ---
    let after_open = &trimmed[4..];
    if let Some(end_pos) = after_open.find("\n---\n") {
        let rest = &after_open[end_pos + 5..];
        // Strip leading blank line after front matter
        rest.strip_prefix('\n').unwrap_or(rest).to_string()
    } else if let Some(end_pos) = after_open.find("\n---\r\n") {
        let rest = &after_open[end_pos + 6..];
        rest.strip_prefix('\n').unwrap_or(rest).to_string()
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_variable_expansion() {
        let content = "Hello {{name}}, version {{version}}";
        let vars = vec![
            ("name".to_string(), "world".to_string()),
            ("version".to_string(), "1.0".to_string()),
        ];
        let result = expand_template(content, Path::new("."), &vars).unwrap();
        assert_eq!(result, "Hello world, version 1.0");
    }

    #[test]
    fn test_include_expansion() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join("child.md"), "child content").unwrap();
        let template = "before\n{{include child.md}}\nafter";
        let result = expand_template(template, root, &[]).unwrap();
        assert_eq!(result, "before\nchild content\nafter");
    }

    #[test]
    fn test_circular_include_detection() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join("a.md"), "{{include b.md}}").unwrap();
        fs::write(root.join("b.md"), "{{include a.md}}").unwrap();
        let result = expand_template("{{include a.md}}", root, &[]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Circular include"),
            "Expected circular include error, got: {err}"
        );
    }

    #[test]
    fn test_nested_includes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join("child.md"), "child").unwrap();
        fs::write(root.join("parent.md"), "parent-{{include child.md}}").unwrap();
        let result = expand_template("{{include parent.md}}", root, &[]).unwrap();
        assert_eq!(result, "parent-child");
    }
}

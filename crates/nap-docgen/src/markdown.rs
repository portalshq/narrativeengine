use crate::model::{ArgModel, CommandModel, DocMeta, WorkspaceMeta};
use crate::util;
use indexmap::IndexMap;

fn yaml_scalar_quote(v: &str) -> String {
    let needs_quote = v.is_empty()
        || v.starts_with('{')
        || v.starts_with('[')
        || v.starts_with('"')
        || v.starts_with('\'')
        || v.starts_with('*')
        || v.starts_with('&')
        || v.starts_with('!')
        || v.starts_with('%')
        || v.starts_with('@')
        || v.starts_with('`')
        || v.contains(':')
        || v.contains('#')
        || v.contains(',')
        || v.contains(' ')
        || v == "true"
        || v == "false"
        || v == "null"
        || v.parse::<f64>().is_ok();
    if needs_quote {
        let escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    } else {
        v.to_string()
    }
}

pub struct MarkdownDoc {
    sections: Vec<Section>,
}

enum Section {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph(String),
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    CodeBlock {
        lang: String,
        code: String,
    },
    List {
        items: Vec<String>,
        ordered: bool,
    },
    FrontMatter(IndexMap<String, String>),
    Raw(String),
    #[allow(dead_code)]
    HorizontalRule,
}

impl MarkdownDoc {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    pub fn front_matter(&mut self, meta: &DocMeta) {
        let mut fm = IndexMap::new();
        fm.insert("generated".to_string(), "true".to_string());
        fm.insert("generator".to_string(), meta.generator_name.clone());
        fm.insert("version".to_string(), meta.crate_version.clone());
        fm.insert("source".to_string(), "clap".to_string());
        self.sections.push(Section::FrontMatter(fm));
    }

    pub fn heading(&mut self, level: u8, text: &str) {
        self.sections.push(Section::Heading {
            level,
            text: text.to_string(),
        });
    }

    pub fn paragraph(&mut self, text: &str) {
        self.sections.push(Section::Paragraph(text.to_string()));
    }

    pub fn table(&mut self, headers: &[&str], rows: Vec<Vec<String>>) {
        self.sections.push(Section::Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows,
        });
    }

    pub fn code_block(&mut self, lang: &str, code: &str) {
        self.sections.push(Section::CodeBlock {
            lang: lang.to_string(),
            code: code.to_string(),
        });
    }

    pub fn list(&mut self, items: &[&str], ordered: bool) {
        self.sections.push(Section::List {
            items: items.iter().map(|s| s.to_string()).collect(),
            ordered,
        });
    }

    #[allow(dead_code)]
    pub fn horizontal_rule(&mut self) {
        self.sections.push(Section::HorizontalRule);
    }

    pub fn raw(&mut self, md: &str) {
        self.sections.push(Section::Raw(md.to_string()));
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        let mut prev_was_heading = false;

        for section in &self.sections {
            match section {
                Section::FrontMatter(fm) => {
                    out.push_str("---\n");
                    for (k, v) in fm {
                        let safe_val = yaml_scalar_quote(v);
                        out.push_str(&format!("{k}: {safe_val}\n"));
                    }
                    out.push_str("---\n\n");
                }
                Section::Heading { level, text } => {
                    if !out.is_empty() && !prev_was_heading {
                        out.push('\n');
                    }
                    let hashes = "#".repeat(*level as usize);
                    out.push_str(&format!("{hashes} {text}\n"));
                    prev_was_heading = true;
                }
                Section::Paragraph(text) => {
                    if prev_was_heading {
                        prev_was_heading = false;
                    }
                    out.push_str(text);
                    if !text.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push('\n');
                }
                Section::Table { headers, rows } => {
                    if prev_was_heading {
                        prev_was_heading = false;
                    }
                    out.push('\n');
                    out.push('|');
                    for h in headers {
                        out.push_str(&format!(" {} |", util::escape_markdown(h)));
                    }
                    out.push('\n');
                    out.push('|');
                    for _ in headers {
                        out.push_str("---|");
                    }
                    out.push('\n');
                    for row in rows {
                        out.push('|');
                        for cell in row {
                            out.push_str(&format!(" {} |", util::escape_markdown(cell)));
                        }
                        out.push('\n');
                    }
                    out.push('\n');
                }
                Section::CodeBlock { lang, code } => {
                    if prev_was_heading {
                        prev_was_heading = false;
                    }
                    out.push_str(&format!("```{lang}\n{code}\n```\n\n"));
                }
                Section::List { items, ordered } => {
                    if prev_was_heading {
                        prev_was_heading = false;
                    }
                    for (i, item) in items.iter().enumerate() {
                        if *ordered {
                            out.push_str(&format!("{}. {}\n", i + 1, item));
                        } else {
                            out.push_str(&format!("- {}\n", item));
                        }
                    }
                    out.push('\n');
                }
                Section::HorizontalRule => {
                    if !prev_was_heading && !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str("---\n");
                    prev_was_heading = false;
                }
                Section::Raw(text) => {
                    out.push_str(text);
                    if !text.ends_with('\n') {
                        out.push('\n');
                    }
                    prev_was_heading = false;
                }
            }
        }

        let out = util::normalize_line_endings(&out);
        util::ensure_trailing_newline(&out)
    }
}

pub fn render_command_page(
    cmd: &CommandModel,
    meta: &DocMeta,
    example_snippet: Option<&str>,
) -> String {
    let mut doc = MarkdownDoc::new();

    doc.front_matter(meta);

    doc.heading(1, &format!("nap {}", cmd.full_path));

    if !cmd.about.is_empty() {
        doc.paragraph(&cmd.about);
    }

    doc.heading(2, "Synopsis");
    let synopsis = cmd.usage.strip_prefix("Usage: ").unwrap_or(&cmd.usage);
    doc.code_block("bash", &format!("nap {synopsis}"));

    if let Some(ref long) = cmd.long_about {
        doc.heading(2, "Description");
        doc.paragraph(long);
    }

    if !cmd.arguments.is_empty() {
        doc.heading(2, "Arguments");
        let mut rows: Vec<Vec<String>> = cmd
            .arguments
            .iter()
            .map(|a| {
                vec![
                    a.name.clone(),
                    a.about.clone(),
                    if a.required {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    },
                ]
            })
            .collect();
        rows.sort_by(|a, b| a[0].cmp(&b[0]));
        doc.table(&["Name", "Description", "Required"], rows);
    }

    if !cmd.options.is_empty() {
        doc.heading(2, "Options");
        let mut rows: Vec<Vec<String>> = cmd
            .options
            .iter()
            .map(|o| {
                let flag = match (o.short, o.long.as_deref()) {
                    (Some(s), Some(l)) => format!("-{s}, --{l}"),
                    (None, Some(l)) => format!("    --{l}"),
                    (Some(s), None) => format!("-{s}"),
                    (None, None) => o.name.clone(),
                };
                vec![
                    flag,
                    o.about.clone(),
                    o.default_value.clone().unwrap_or_default(),
                ]
            })
            .collect();
        rows.sort_by(|a, b| a[0].cmp(&b[0]));
        doc.table(&["Flag", "Description", "Default"], rows);
    }

    if !cmd.flags.is_empty() {
        doc.heading(2, "Flags");
        let mut rows: Vec<Vec<String>> = cmd
            .flags
            .iter()
            .map(|f| {
                let flag = match (f.short, f.long.as_deref()) {
                    (Some(s), Some(l)) => format!("-{s}, --{l}"),
                    (None, Some(l)) => format!("    --{l}"),
                    (Some(s), None) => format!("-{s}"),
                    (None, None) => f.name.clone(),
                };
                vec![flag, f.about.clone()]
            })
            .collect();
        rows.sort_by(|a, b| a[0].cmp(&b[0]));
        doc.table(&["Flag", "Description"], rows);
    }

    if !cmd.env_vars.is_empty() {
        doc.heading(2, "Environment Variables");
        let mut rows: Vec<Vec<String>> = cmd
            .env_vars
            .iter()
            .map(|e| vec![e.name.clone(), e.about.clone()])
            .collect();
        rows.sort_by(|a, b| a[0].cmp(&b[0]));
        doc.table(&["Variable", "Description"], rows);
    }

    if !cmd.visible_aliases.is_empty() {
        doc.heading(2, "Aliases");
        doc.list(
            &cmd.visible_aliases
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            false,
        );
    }

    if !cmd.subcommands.is_empty() {
        doc.heading(2, "Subcommands");
        let mut rows: Vec<Vec<String>> = cmd
            .subcommands
            .iter()
            .map(|s| vec![s.name.clone(), s.about.clone()])
            .collect();
        rows.sort_by(|a, b| a[0].cmp(&b[0]));
        doc.table(&["Command", "Description"], rows);
    }

    if let Some(snippet) = example_snippet {
        doc.heading(2, "Examples");
        doc.raw(snippet);
    }

    doc.heading(2, "Source");
    doc.paragraph(&format!(
        "`crates/nap-cli/src/main.rs` — `{}` command",
        cmd.name
    ));

    doc.render()
}

pub fn render_command_index(commands: &[CommandModel], meta: &DocMeta) -> String {
    let mut doc = MarkdownDoc::new();
    doc.front_matter(meta);
    doc.heading(1, "CLI Command Reference");
    doc.paragraph("Complete reference for all `nap` CLI commands.");

    let mut rows: Vec<Vec<String>> = commands
        .iter()
        .map(|c| {
            vec![
                format!(
                    "[`nap {}`](docs/generated/commands/{}.md)",
                    c.full_path, c.name
                ),
                c.about.clone(),
            ]
        })
        .collect();
    rows.sort_by(|a, b| a[0].cmp(&b[0]));
    doc.table(&["Command", "Description"], rows);

    doc.render()
}

pub fn render_cli_summary(
    commands: &[CommandModel],
    cargo_meta: &WorkspaceMeta,
    global_options: &[ArgModel],
    meta: &DocMeta,
) -> String {
    let mut doc = MarkdownDoc::new();
    doc.front_matter(meta);

    doc.heading(1, "NAP CLI Reference");
    doc.paragraph(&format!(
        "The `nap` command-line interface (v{}) provides tools for creating, resolving, and managing narrative resources using the Narrative Addressing Protocol.",
        cargo_meta.version
    ));

    doc.heading(2, "Command Overview");
    let mut rows: Vec<Vec<String>> = commands
        .iter()
        .map(|c| {
            vec![
                format!(
                    "[`nap {}`](docs/generated/commands/{}.md)",
                    c.full_path, c.name
                ),
                c.about.clone(),
            ]
        })
        .collect();
    rows.sort_by(|a, b| a[0].cmp(&b[0]));
    doc.table(&["Command", "Description"], rows);

    doc.heading(2, "Global Options");
    let mut opt_rows: Vec<Vec<String>> = global_options
        .iter()
        .map(|o| {
            let flag = match (o.short, o.long.as_deref()) {
                (Some(s), Some(l)) => format!("-{s}, --{l}"),
                (None, Some(l)) => format!("    --{l}"),
                (Some(s), None) => format!("-{s}"),
                (None, None) => o.name.clone(),
            };
            let value = o
                .value_name
                .as_ref()
                .map(|v| format!(" <{v}>"))
                .unwrap_or_default();
            vec![
                format!("{flag}{value}"),
                o.about.clone(),
                o.default_value.clone().unwrap_or_default(),
            ]
        })
        .collect();
    opt_rows.sort_by(|a, b| a[0].cmp(&b[0]));
    doc.table(&["Flag", "Description", "Default"], opt_rows);

    doc.heading(2, "Output Formats");
    doc.paragraph(
        "Most commands support `--format` (`-f`) with values `yaml` (default) or `json`.",
    );
    doc.paragraph(
        "When stdout is not a terminal, JSON is used automatically. Override with `$NAP_OUTPUT`.",
    );

    doc.heading(2, "Common Examples");
    doc.code_block(
        "bash",
        "# Initialize a repository\nnap init starwars\n\n# Create an entity\nnap create character lukeskywalker -u starwars -n \"Luke Skywalker\"\n\n# Resolve a manifest\nnap resolve nap://starwars/character/lukeskywalker\n\n# Query a subtree\nnap query nap://starwars/character/lukeskywalker properties\n\n# View commit history\nnap history nap://starwars/character/lukeskywalker",
    );

    doc.render()
}

pub fn render_global_options(options: &[ArgModel], meta: &DocMeta) -> String {
    let mut doc = MarkdownDoc::new();
    doc.front_matter(meta);
    doc.heading(1, "Global Options");

    doc.paragraph("These options are available on all `nap` commands.");

    let mut rows: Vec<Vec<String>> = options
        .iter()
        .map(|o| {
            let flag = match (o.short, o.long.as_deref()) {
                (Some(s), Some(l)) => format!("-{s}, --{l}"),
                (None, Some(l)) => format!("    --{l}"),
                (Some(s), None) => format!("-{s}"),
                (None, None) => o.name.clone(),
            };
            let value = o
                .value_name
                .as_ref()
                .map(|v| format!(" <{v}>"))
                .unwrap_or_default();
            vec![
                format!("{flag}{value}"),
                o.about.clone(),
                o.default_value.clone().unwrap_or_default(),
            ]
        })
        .collect();
    rows.sort_by(|a, b| a[0].cmp(&b[0]));
    doc.table(&["Flag", "Description", "Default"], rows);

    doc.render()
}

pub fn render_environment_page(commands: &[CommandModel], meta: &DocMeta) -> String {
    let mut doc = MarkdownDoc::new();
    doc.front_matter(meta);
    doc.heading(1, "Environment Variables");

    doc.paragraph("The following environment variables are recognized by `nap`.");

    let mut vars: IndexMap<String, String> = IndexMap::new();
    collect_env_vars(commands, &mut vars);

    let mut rows: Vec<Vec<String>> = vars
        .iter()
        .map(|(name, desc)| vec![name.clone(), desc.clone()])
        .collect();
    rows.sort_by(|a, b| a[0].cmp(&b[0]));
    doc.table(&["Variable", "Description"], rows);

    doc.render()
}

fn collect_env_vars(commands: &[CommandModel], out: &mut IndexMap<String, String>) {
    for cmd in commands {
        for env in &cmd.env_vars {
            out.entry(env.name.clone())
                .or_insert_with(|| env.about.clone());
        }
        collect_env_vars(&cmd.subcommands, out);
    }
}

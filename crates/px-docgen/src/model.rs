#[derive(Debug, Clone)]
pub struct DocMeta {
    pub generator_version: String,
    pub crate_version: String,
    pub generator_name: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommandModel {
    pub name: String,
    pub parent_path: Option<String>,
    pub full_path: String,
    pub about: String,
    pub long_about: Option<String>,
    pub usage: String,
    pub aliases: Vec<String>,
    pub visible_aliases: Vec<String>,
    pub arguments: Vec<ArgModel>,
    pub options: Vec<ArgModel>,
    pub flags: Vec<ArgModel>,
    pub subcommands: Vec<CommandModel>,
    pub hidden: bool,
    pub deprecated: bool,
    pub env_vars: Vec<EnvVarModel>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArgModel {
    pub name: String,
    pub about: String,
    pub long_about: Option<String>,
    pub required: bool,
    pub takes_value: bool,
    pub default_value: Option<String>,
    pub value_name: Option<String>,
    pub possible_values: Vec<String>,
    pub short: Option<char>,
    pub long: Option<String>,
    pub multiple: bool,
    pub hidden: bool,
    pub env: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnvVarModel {
    pub name: String,
    pub about: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceMeta {
    pub name: String,
    pub version: String,
    pub license: String,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub description: String,
}

#[allow(dead_code)]
impl CommandModel {
    pub fn flatten_all(&self) -> Vec<&CommandModel> {
        let mut result = vec![self];
        for child in &self.subcommands {
            result.extend(child.flatten_all());
        }
        result
    }

    pub fn sorted_subcommands(&self) -> Vec<&CommandModel> {
        let mut subs: Vec<&CommandModel> = self.subcommands.iter().collect();
        subs.sort_by(|a, b| a.name.cmp(&b.name));
        subs
    }
}

use crate::model::CommandModel;

pub fn render_command_graph(commands: &[CommandModel]) -> String {
    let mut mermaid = String::new();
    mermaid.push_str("graph LR\n");

    for cmd in commands {
        render_command_node(cmd, &mut mermaid, "nap");
    }

    mermaid
}

fn render_command_node(cmd: &CommandModel, out: &mut String, parent: &str) {
    let node_id = format!("{}_{}", parent, cmd.name.replace(' ', "_"));

    let label = format!("nap {}", cmd.full_path);

    out.push_str(&format!(
        "    {parent}[\"{parent}\"] --> {node_id}[\"{label}\"]\n"
    ));

    for sub in &cmd.subcommands {
        render_command_node(sub, out, &node_id);
    }
}

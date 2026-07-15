use crate::model::CommandModel;

pub fn render_command_graph(commands: &[CommandModel]) -> String {
    let mut dot = String::new();
    dot.push_str("digraph nap_commands {\n");
    dot.push_str("    rankdir=LR;\n");
    dot.push_str("    node [shape=box, style=filled, fillcolor=lightblue];\n");
    dot.push_str("    \n");

    for cmd in commands {
        render_command_node(cmd, &mut dot, "nap");
    }

    dot.push_str("}\n");
    dot
}

fn render_command_node(cmd: &CommandModel, out: &mut String, parent: &str) {
    let node_id = if cmd.parent_path.is_some() {
        format!("{}_{}", parent, cmd.name.replace(' ', "_"))
    } else {
        cmd.name.clone()
    };

    let label = if cmd.full_path.contains(' ') {
        format!("nap {}", cmd.full_path)
    } else {
        format!("nap {}", cmd.name)
    };

    out.push_str(&format!("    \"{}\" [label=\"{}\"];\n", node_id, label));
    out.push_str(&format!("    \"{parent}\" -> \"{node_id}\";\n"));

    for sub in &cmd.subcommands {
        render_command_node(sub, out, &node_id);
    }
}

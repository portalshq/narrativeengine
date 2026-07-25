use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=../../docs/generated/commands.json");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let commands_path = manifest_dir.join("../../docs/generated/commands.json");
    let commands: Value =
        serde_json::from_str(&fs::read_to_string(commands_path).expect("read commands.json"))
            .expect("parse commands.json");

    let mut tools = Vec::new();
    for command in commands["commands"].as_array().expect("commands array") {
        collect_tools(command, &mut tools);
    }

    let json = serde_json::to_string(&tools).expect("serialize generated tools");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    fs::write(
        out_dir.join("generated_tools.rs"),
        format!("pub const GENERATED_TOOLS_JSON: &str = {json:?};\n"),
    )
    .expect("write generated tools");
}

fn collect_tools(command: &Value, tools: &mut Vec<Value>) {
    if command["hidden"].as_bool().unwrap_or(false) {
        return;
    }

    tools.push(tool_for(command));

    if let Some(subcommands) = command["subcommands"].as_array() {
        for subcommand in subcommands {
            collect_tools(subcommand, tools);
        }
    }
}

fn tool_for(command: &Value) -> Value {
    let full_path = command["full_path"].as_str().unwrap();
    let about = command["about"].as_str().unwrap_or(full_path);
    let subcommands = command["subcommands"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let is_parent = !subcommands.is_empty();
    let command_parts: Vec<Value> = full_path
        .split_whitespace()
        .map(|part| Value::String(part.to_string()))
        .chain(is_parent.then(|| Value::String("--help".to_string())))
        .collect();

    let mut params = Vec::new();
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for arg in command["arguments"].as_array().unwrap_or(&Vec::new()) {
        let name = param_name(arg);
        properties.insert(name.clone(), schema_for_param(arg, "argument"));
        if arg["required"].as_bool().unwrap_or(false) {
            required.push(Value::String(name.clone()));
        }
        params.push(json!({
            "name": name,
            "kind": "argument",
            "cli_name": Value::Null,
            "required": arg["required"].as_bool().unwrap_or(false),
            "output_format": false
        }));
    }

    for opt in command["options"].as_array().unwrap_or(&Vec::new()) {
        if opt["name"].as_str() == Some("help") {
            continue;
        }
        let name = param_name(opt);
        properties.insert(name.clone(), schema_for_param(opt, "option"));
        if opt["required"].as_bool().unwrap_or(false) {
            required.push(Value::String(name.clone()));
        }
        params.push(json!({
            "name": name,
            "kind": "option",
            "cli_name": format!("--{}", opt["long"].as_str().unwrap_or(opt["name"].as_str().unwrap()).replace('_', "-")),
            "required": opt["required"].as_bool().unwrap_or(false),
            "output_format": is_output_format(opt)
        }));
    }

    for flag in command["flags"].as_array().unwrap_or(&Vec::new()) {
        if flag["name"].as_str() == Some("help") {
            continue;
        }
        let name = param_name(flag);
        properties.insert(name.clone(), schema_for_param(flag, "flag"));
        params.push(json!({
            "name": name,
            "kind": "flag",
            "cli_name": format!("--{}", flag["long"].as_str().unwrap_or(flag["name"].as_str().unwrap()).replace('_', "-")),
            "required": false,
            "output_format": false
        }));
    }

    let input_schema = json!({
        "type": "object",
        "properties": Value::Object(properties),
        "required": required,
        "additionalProperties": false
    });

    json!({
        "name": format!("nap_{}", full_path.replace(['-', ' '], "_")),
        "title": full_path,
        "description": description(command, about),
        "input_schema": input_schema,
        "annotations": annotations(full_path, is_parent),
        "command": command_parts,
        "params": params,
        "timeout_ms": timeout_ms(full_path)
    })
}

fn param_name(param: &Value) -> String {
    param["name"].as_str().unwrap().replace('-', "_")
}

fn schema_for_param(param: &Value, kind: &str) -> Value {
    let mut schema = serde_json::Map::new();
    schema.insert(
        "type".to_string(),
        Value::String(
            if kind == "flag" {
                "boolean"
            } else if param["name"].as_str().unwrap_or("").ends_with("limit")
                || param["name"].as_str() == Some("limit")
            {
                "integer"
            } else {
                "string"
            }
            .to_string(),
        ),
    );

    if let Some(about) = param["about"].as_str() {
        schema.insert("description".to_string(), Value::String(about.to_string()));
    }

    let values = possible_values(param);
    if !values.is_empty() {
        schema.insert(
            "enum".to_string(),
            Value::Array(values.into_iter().map(Value::String).collect()),
        );
        schema.insert("type".to_string(), Value::String("string".to_string()));
    }

    if kind == "flag" {
        schema.insert("default".to_string(), Value::Bool(false));
    } else if is_output_format(param) {
        schema.insert("default".to_string(), Value::String("json".to_string()));
    } else if let Some(default) = param["default_value"].as_str() {
        if schema.get("type").and_then(Value::as_str) == Some("integer") {
            if let Ok(number) = default.parse::<i64>() {
                schema.insert("default".to_string(), Value::Number(number.into()));
            }
        } else {
            schema.insert("default".to_string(), Value::String(default.to_string()));
        }
    }

    Value::Object(schema)
}

fn possible_values(param: &Value) -> Vec<String> {
    if let Some(values) = param["possible_values"].as_array() {
        return values
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect();
    }

    let name = param["name"].as_str().unwrap_or("");
    let about = param["about"].as_str().unwrap_or("").to_lowercase();

    if name == "format" && about.contains("json") && about.contains("yaml") {
        return vec!["json".to_string(), "yaml".to_string()];
    }
    if (name == "provider" || name == "entity_type")
        && about.contains("local")
        && about.contains("portals-cloud")
        && about.contains("remote")
    {
        return vec![
            "local".to_string(),
            "portals-cloud".to_string(),
            "remote".to_string(),
        ];
    }
    if name == "name" && about.contains("manifest") && about.contains("commit") {
        return vec!["manifest".to_string(), "commit".to_string()];
    }

    Vec::new()
}

fn is_output_format(param: &Value) -> bool {
    param["name"].as_str() == Some("format")
        && param["about"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .contains("output format")
}

fn description(command: &Value, about: &str) -> String {
    let mut parts = Vec::new();
    for key in ["arguments", "options", "flags"] {
        for param in command[key].as_array().unwrap_or(&Vec::new()) {
            if param["name"].as_str() == Some("help") {
                continue;
            }
            let display = param["long"]
                .as_str()
                .map(|long| format!("--{long}"))
                .unwrap_or_else(|| param["name"].as_str().unwrap().to_string());
            parts.push(if param["required"].as_bool().unwrap_or(false) {
                format!("{display} (required)")
            } else {
                display
            });
        }
    }

    let args = if parts.is_empty() {
        "Args: none".to_string()
    } else {
        format!("Args: {}", parts.join(", "))
    };

    let mut text = format!("{about}\n\n{args}");
    let subcommands: BTreeSet<String> = command["subcommands"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|cmd| cmd["name"].as_str().map(ToString::to_string))
        .collect();
    if !subcommands.is_empty() {
        text.push_str(&format!(
            "\n\nSubcommands: {}",
            subcommands.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    text
}

fn annotations(full_path: &str, is_parent: bool) -> Value {
    let base = full_path.split_whitespace().next().unwrap_or(full_path);
    let read_only = matches!(
        base,
        "resolve"
            | "query"
            | "history"
            | "list"
            | "head-hash"
            | "status"
            | "schema"
            | "validate"
            | "verify"
            | "diff"
            | "content-hash"
    );
    let destructive = matches!(base, "delete" | "revert");
    json!({
        "readOnlyHint": is_parent || read_only,
        "destructiveHint": !is_parent && destructive,
        "idempotentHint": is_parent || read_only || matches!(base, "status" | "schema"),
        "openWorldHint": false
    })
}

fn timeout_ms(full_path: &str) -> u64 {
    match full_path.split_whitespace().next().unwrap_or(full_path) {
        "init" | "install" => 120_000,
        "doctor" => 60_000,
        _ => 30_000,
    }
}

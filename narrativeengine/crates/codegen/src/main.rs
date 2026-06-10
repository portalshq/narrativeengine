use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, value_enum)]
    language: Language,

    #[arg(long)]
    out: PathBuf,
}

#[derive(Clone, Debug, ValueEnum)]
enum Language {
    Python,
    Typescript,
    Go,
    Java,
    Csharp,
    Swift,
    Kotlin,
}

#[derive(Clone, Debug)]
struct Field {
    name: String,
    type_name: String,
    required: bool,
}

#[derive(Clone, Debug)]
struct Model {
    name: String,
    fields: Vec<Field>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let models = read_models()?;
    let contents = match args.language {
        Language::Python => render_python(&models),
        Language::Typescript => render_typescript(&models),
        Language::Go => render_go(&models),
        Language::Java => render_java(&models),
        Language::Csharp => render_csharp(&models),
        Language::Swift => render_swift(&models),
        Language::Kotlin => render_kotlin(&models),
    };

    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&args.out, contents)
        .with_context(|| format!("failed to write {}", args.out.display()))?;
    Ok(())
}

fn read_models() -> Result<Vec<Model>> {
    let bundle = narrativeengine_core::schema_bundle();
    let schemas = bundle
        .get("models")
        .and_then(Value::as_object)
        .context("schema bundle is missing models")?;
    let field_order = bundle
        .get("field_order")
        .and_then(Value::as_object)
        .context("schema bundle is missing field_order")?;

    narrativeengine_core::MODEL_NAMES
        .iter()
        .map(|name| {
            let schema = schemas
                .get(*name)
                .with_context(|| format!("schema bundle is missing {name}"))?;
            let order = field_order
                .get(*name)
                .and_then(Value::as_array)
                .with_context(|| format!("schema bundle is missing field order for {name}"))?;
            parse_model(name, schema, order)
        })
        .collect()
}

fn parse_model(name: &str, schema: &Value, order: &[Value]) -> Result<Model> {
    let root = schema
        .as_object()
        .context("model schema is missing root schema")?;
    let required = root
        .get("required")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let properties = root
        .get("properties")
        .and_then(Value::as_object)
        .context("model schema is missing properties")?;

    let mut fields = Vec::new();
    for field_name in order.iter().filter_map(Value::as_str) {
        let field_schema = properties
            .get(field_name)
            .with_context(|| format!("{name}.{field_name} is missing from schema properties"))?;
        fields.push(Field {
            name: field_name.to_string(),
            type_name: schema_type(field_schema)?,
            required: required.iter().any(|item| item == field_name),
        });
    }

    Ok(Model {
        name: name.to_string(),
        fields,
    })
}

fn schema_type(schema: &Value) -> Result<String> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return Ok(reference
            .rsplit('/')
            .next()
            .context("invalid $ref")?
            .to_string());
    }

    if schema.get("type").and_then(Value::as_str) == Some("array") {
        let item_type = schema
            .get("items")
            .map(schema_type)
            .transpose()?
            .unwrap_or_else(|| "Any".to_string());
        return Ok(format!("Vec<{item_type}>"));
    }

    match schema.get("type").and_then(Value::as_str) {
        Some("string") => Ok("String".to_string()),
        Some("integer") => match schema.get("format").and_then(Value::as_str) {
            Some("uint32") => Ok("u32".to_string()),
            _ => Ok("u64".to_string()),
        },
        Some("number") => Ok("f64".to_string()),
        Some("boolean") => Ok("bool".to_string()),
        other => bail!("unsupported schema type: {other:?}"),
    }
}

fn header() -> &'static str {
    "/* This file is generated from Rust schemas by narrativeengine-codegen. */\n"
}

fn render_python(models: &[Model]) -> String {
    let mut out = String::from(
        "# This file is generated from Rust schemas by narrativeengine-codegen.\n\
from __future__ import annotations\n\n\
from typing import Any\n\n\
try:\n    from pydantic import BaseModel, ConfigDict\n    _HAS_PYDANTIC = True\n\
except ImportError:\n    BaseModel = object  # type: ignore[assignment]\n    ConfigDict = dict  # type: ignore[assignment]\n    _HAS_PYDANTIC = False\n\n\n\
if _HAS_PYDANTIC:\n",
    );

    for model in models {
        out.push_str(&format!("    class {}(BaseModel):\n", model.name));
        out.push_str("        model_config = ConfigDict(extra=\"forbid\")\n");
        for field in &model.fields {
            out.push_str(&format!(
                "        {}: {}\n",
                field.name,
                python_type(&field.type_name, field.required)
            ));
        }
        out.push('\n');
    }

    out.push_str("else:\n");
    for model in models {
        out.push_str(&format!("    {} = dict[str, Any]\n", model.name));
    }

    out.push_str("\n\n__all__ = [\n");
    for model in models {
        out.push_str(&format!("    \"{}\",\n", model.name));
    }
    out.push_str("]\n");
    out
}

fn render_typescript(models: &[Model]) -> String {
    let mut out = String::from(header());
    out.push('\n');
    for model in models {
        out.push_str(&format!("export interface {} {{\n", model.name));
        for field in &model.fields {
            out.push_str(&format!(
                "  {}{}: {};\n",
                field.name,
                if field.required { "" } else { "?" },
                typescript_type(&field.type_name)
            ));
        }
        out.push_str("}\n\n");
    }
    out
}

fn render_go(models: &[Model]) -> String {
    let mut out = String::from("// This file is generated from Rust schemas by narrativeengine-codegen.\npackage narrativeengine\n\n");
    for model in models {
        out.push_str(&format!("type {} struct {{\n", model.name));
        for field in &model.fields {
            out.push_str(&format!(
                "\t{} {} `json:\"{}\"`\n",
                pascal_case(&field.name),
                go_type(&field.type_name),
                field.name
            ));
        }
        out.push_str("}\n\n");
    }
    out
}

fn render_java(models: &[Model]) -> String {
    let mut out = String::from("// This file is generated from Rust schemas by narrativeengine-codegen.\npackage com.narrativeengine;\n\nimport java.util.List;\n\n");
    out.push_str("public final class NarrativeModels {\n    private NarrativeModels() {}\n\n");
    for model in models {
        out.push_str(&format!("    public record {}(\n", model.name));
        for (index, field) in model.fields.iter().enumerate() {
            out.push_str(&format!(
                "        {} {}{}\n",
                java_type(&field.type_name),
                field.name,
                if index + 1 == model.fields.len() {
                    ""
                } else {
                    ","
                }
            ));
        }
        out.push_str("    ) {}\n\n");
    }
    out.push_str("}\n");
    out
}

fn render_csharp(models: &[Model]) -> String {
    let mut out = String::from("// This file is generated from Rust schemas by narrativeengine-codegen.\nusing System.Collections.Generic;\n\nnamespace NarrativeEngine;\n\n");
    for model in models {
        out.push_str(&format!("public sealed record {}(\n", model.name));
        for (index, field) in model.fields.iter().enumerate() {
            out.push_str(&format!(
                "    {} {}{}\n",
                csharp_type(&field.type_name),
                pascal_case(&field.name),
                if index + 1 == model.fields.len() {
                    ""
                } else {
                    ","
                }
            ));
        }
        out.push_str(");\n\n");
    }
    out
}

fn render_swift(models: &[Model]) -> String {
    let mut out = String::from("// This file is generated from Rust schemas by narrativeengine-codegen.\nimport Foundation\n\n");
    for model in models {
        out.push_str(&format!(
            "public struct {}: Codable, Equatable {{\n",
            model.name
        ));
        for field in &model.fields {
            out.push_str(&format!(
                "    public let {}: {}\n",
                field.name,
                swift_type(&field.type_name)
            ));
        }
        out.push_str("}\n\n");
    }
    out
}

fn render_kotlin(models: &[Model]) -> String {
    let mut out = String::from("// This file is generated from Rust schemas by narrativeengine-codegen.\npackage com.narrativeengine\n\n");
    for model in models {
        out.push_str(&format!("data class {}(\n", model.name));
        for (index, field) in model.fields.iter().enumerate() {
            out.push_str(&format!(
                "    val {}: {}{}\n",
                field.name,
                kotlin_type(&field.type_name),
                if index + 1 == model.fields.len() {
                    ""
                } else {
                    ","
                }
            ));
        }
        out.push_str(")\n\n");
    }
    out
}

fn python_type(type_name: &str, required: bool) -> String {
    let base = match type_name {
        "String" => "str".to_string(),
        "u64" => "int".to_string(),
        "f64" => "float".to_string(),
        "u32" => "int".to_string(),
        "bool" => "bool".to_string(),
        value if value.starts_with("Vec<") => format!(
            "list[{}]",
            python_type(value.trim_start_matches("Vec<").trim_end_matches('>'), true)
        ),
        value => value.to_string(),
    };

    if required {
        base
    } else {
        format!("{base} | None = None")
    }
}

fn typescript_type(type_name: &str) -> String {
    match type_name {
        "String" => "string".to_string(),
        "u64" | "u32" | "f64" => "number".to_string(),
        "bool" => "boolean".to_string(),
        value if value.starts_with("Vec<") => format!(
            "{}[]",
            typescript_type(value.trim_start_matches("Vec<").trim_end_matches('>'))
        ),
        value => value.to_string(),
    }
}

fn go_type(type_name: &str) -> String {
    match type_name {
        "String" => "string".to_string(),
        "u64" => "uint64".to_string(),
        "u32" => "uint32".to_string(),
        "f64" => "float64".to_string(),
        "bool" => "bool".to_string(),
        value if value.starts_with("Vec<") => format!(
            "[]{}",
            go_type(value.trim_start_matches("Vec<").trim_end_matches('>'))
        ),
        value => value.to_string(),
    }
}

fn java_type(type_name: &str) -> String {
    match type_name {
        "String" => "String".to_string(),
        "u64" => "long".to_string(),
        "u32" => "int".to_string(),
        "f64" => "double".to_string(),
        "bool" => "boolean".to_string(),
        value if value.starts_with("Vec<") => format!(
            "List<{}>",
            java_type(value.trim_start_matches("Vec<").trim_end_matches('>'))
        ),
        value => value.to_string(),
    }
}

fn csharp_type(type_name: &str) -> String {
    match type_name {
        "String" => "string".to_string(),
        "u64" => "ulong".to_string(),
        "u32" => "uint".to_string(),
        "f64" => "double".to_string(),
        "bool" => "bool".to_string(),
        value if value.starts_with("Vec<") => format!(
            "IReadOnlyList<{}>",
            csharp_type(value.trim_start_matches("Vec<").trim_end_matches('>'))
        ),
        value => value.to_string(),
    }
}

fn swift_type(type_name: &str) -> String {
    match type_name {
        "String" => "String".to_string(),
        "u64" | "u32" => "UInt64".to_string(),
        "f64" => "Double".to_string(),
        "bool" => "Bool".to_string(),
        value if value.starts_with("Vec<") => format!(
            "[{}]",
            swift_type(value.trim_start_matches("Vec<").trim_end_matches('>'))
        ),
        value => value.to_string(),
    }
}

fn kotlin_type(type_name: &str) -> String {
    match type_name {
        "String" => "String".to_string(),
        "u64" | "u32" => "Long".to_string(),
        "f64" => "Double".to_string(),
        "bool" => "Boolean".to_string(),
        value if value.starts_with("Vec<") => format!(
            "List<{}>",
            kotlin_type(value.trim_start_matches("Vec<").trim_end_matches('>'))
        ),
        value => value.to_string(),
    }
}

fn pascal_case(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

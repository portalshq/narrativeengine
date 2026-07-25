use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeSet, env, path::PathBuf, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
    time,
};

include!(concat!(env!("OUT_DIR"), "/generated_tools.rs"));

pub const PROTOCOL_VERSION: &str = "2025-06-18";
const MAX_OUTPUT_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneratedTool {
    pub name: String,
    pub title: String,
    pub description: String,
    pub input_schema: Value,
    pub annotations: Value,
    pub command: Vec<String>,
    pub params: Vec<ParamSpec>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParamSpec {
    pub name: String,
    pub kind: ParamKind,
    pub cli_name: Option<String>,
    pub required: bool,
    pub output_format: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParamKind {
    Argument,
    Option,
    Flag,
}

#[derive(Debug)]
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub fn generated_tools() -> Vec<GeneratedTool> {
    serde_json::from_str(GENERATED_TOOLS_JSON).expect("generated MCP tools are valid JSON")
}

pub fn build_cli_args(tool: &GeneratedTool, arguments: &Value) -> Result<Vec<String>> {
    let args_object = arguments
        .as_object()
        .ok_or_else(|| anyhow!("tool arguments must be a JSON object"))?;
    validate_arguments(tool, args_object)?;
    let mut args = tool.command.clone();

    for param in &tool.params {
        let value = args_object.get(&param.name);
        match param.kind {
            ParamKind::Argument => {
                if let Some(value) = value.filter(|value| !value.is_null()) {
                    args.push(value_to_cli_string(value)?);
                } else if param.required {
                    return Err(anyhow!("missing required argument '{}'", param.name));
                }
            }
            ParamKind::Option => {
                let cli_name = param
                    .cli_name
                    .as_ref()
                    .ok_or_else(|| anyhow!("missing CLI name for option '{}'", param.name))?;
                if param.output_format {
                    args.push(cli_name.clone());
                    args.push("json".to_string());
                } else if let Some(value) = value.filter(|value| !value.is_null()) {
                    args.push(cli_name.clone());
                    args.push(value_to_cli_string(value)?);
                } else if param.required {
                    return Err(anyhow!("missing required option '{}'", param.name));
                }
            }
            ParamKind::Flag => {
                if value.and_then(Value::as_bool).unwrap_or(false) {
                    let cli_name = param
                        .cli_name
                        .as_ref()
                        .ok_or_else(|| anyhow!("missing CLI name for flag '{}'", param.name))?;
                    args.push(cli_name.clone());
                }
            }
        }
    }

    Ok(args)
}

fn validate_arguments(
    tool: &GeneratedTool,
    args_object: &serde_json::Map<String, Value>,
) -> Result<()> {
    let known_params: BTreeSet<&str> = tool
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect();
    for key in args_object.keys() {
        if !known_params.contains(key.as_str()) {
            return Err(anyhow!("unknown argument '{key}' for tool '{}'", tool.name));
        }
    }

    for param in &tool.params {
        let Some(value) = args_object.get(&param.name) else {
            if param.required {
                return Err(anyhow!("missing required argument '{}'", param.name));
            }
            continue;
        };
        if value.is_null() {
            if param.required {
                return Err(anyhow!("missing required argument '{}'", param.name));
            }
            continue;
        }

        match param.kind {
            ParamKind::Flag => {
                if !value.is_boolean() {
                    return Err(anyhow!("argument '{}' must be a boolean", param.name));
                }
            }
            ParamKind::Argument | ParamKind::Option => {
                if param.output_format {
                    validate_string_like(&param.name, value)?;
                    continue;
                }
                validate_value_against_schema(&param.name, value, &tool.input_schema)?;
            }
        }
    }

    Ok(())
}

fn validate_value_against_schema(name: &str, value: &Value, input_schema: &Value) -> Result<()> {
    let schema = input_schema
        .get("properties")
        .and_then(|properties| properties.get(name))
        .unwrap_or(&Value::Null);

    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        let Some(value_str) = value.as_str() else {
            return Err(anyhow!("argument '{name}' must be a string"));
        };
        let valid = values
            .iter()
            .filter_map(Value::as_str)
            .any(|candidate| candidate == value_str);
        if !valid {
            return Err(anyhow!(
                "argument '{name}' has unsupported value '{value_str}'"
            ));
        }
    }

    match schema
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("string")
    {
        "integer" => {
            if value.as_i64().is_none() {
                return Err(anyhow!("argument '{name}' must be an integer"));
            }
        }
        "boolean" => {
            if !value.is_boolean() {
                return Err(anyhow!("argument '{name}' must be a boolean"));
            }
        }
        _ => validate_string_like(name, value)?,
    }

    Ok(())
}

fn validate_string_like(name: &str, value: &Value) -> Result<()> {
    if value.is_string() || value.is_number() || value.is_boolean() {
        Ok(())
    } else {
        Err(anyhow!(
            "argument '{name}' must be a string, number, or boolean"
        ))
    }
}

fn value_to_cli_string(value: &Value) -> Result<String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Null => Err(anyhow!("null cannot be converted to a CLI argument")),
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string(value).context("serialize argument")
        }
    }
}

pub async fn execute_tool(tool: &GeneratedTool, arguments: &Value) -> Result<CliOutput> {
    let binary = resolve_nap_binary().await;
    execute_tool_with_binary(tool, arguments, &binary).await
}

pub async fn execute_tool_with_binary(
    tool: &GeneratedTool,
    arguments: &Value,
    binary: &str,
) -> Result<CliOutput> {
    let args = build_cli_args(tool, arguments)?;
    let mut command = Command::new(binary);
    command
        .args(args)
        .env(
            "NAP_OUTPUT",
            env::var("NAP_OUTPUT").unwrap_or_else(|_| "json".to_string()),
        )
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().context("failed to execute nap CLI")?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture nap stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("failed to capture nap stderr"))?;

    let stdout_task = tokio::spawn(read_limited(stdout, MAX_OUTPUT_BYTES));
    let stderr_task = tokio::spawn(read_limited(stderr, MAX_OUTPUT_BYTES));

    let status = match time::timeout(Duration::from_millis(tool.timeout_ms), child.wait()).await {
        Ok(status) => status.context("failed to wait for nap CLI")?,
        Err(_) => {
            let _ = child.kill().await;
            return Err(anyhow!("nap command timed out after {}ms", tool.timeout_ms));
        }
    };

    let stdout = stdout_task
        .await
        .context("stdout reader task failed")?
        .context("failed to read nap stdout")?;
    let stderr = stderr_task
        .await
        .context("stderr reader task failed")?
        .context("failed to read nap stderr")?;

    Ok(CliOutput {
        stdout,
        stderr,
        success: status.success(),
    })
}

async fn read_limited<R>(mut reader: R, limit: usize) -> Result<String>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        if buffer.len() + read > limit {
            let remaining = limit.saturating_sub(buffer.len());
            buffer.extend_from_slice(&chunk[..remaining]);
            return Err(anyhow!("nap output exceeded {limit} bytes"));
        }
        buffer.extend_from_slice(&chunk[..read]);
    }

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

async fn resolve_nap_binary() -> String {
    if let Ok(path) = env::var("NAP_CLI_PATH") {
        return path;
    }

    if let Some(path) = find_in_path("nap") {
        return path.to_string_lossy().to_string();
    }

    let mut candidates = vec![
        PathBuf::from("/opt/homebrew/bin/nap"),
        PathBuf::from("/usr/local/bin/nap"),
    ];
    if let Some(home) = env::var_os("HOME") {
        candidates.push(PathBuf::from(&home).join(".cargo/bin/nap"));
        candidates.push(PathBuf::from(home).join(".local/bin/nap"));
    }

    for candidate in candidates {
        if candidate.is_file() {
            return candidate.to_string_lossy().to_string();
        }
    }

    "nap".to_string()
}

fn find_in_path(binary: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let candidate = dir.join(format!("{binary}.exe"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub async fn handle_message(message: Value, tools: &[GeneratedTool]) -> Option<Value> {
    let id = message.get("id").cloned();
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        return Some(error_response(
            id.unwrap_or(Value::Null),
            -32600,
            "Invalid request",
        ));
    };

    match method {
        "initialize" => Some(response(
            id?,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "nap-mcp-server",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )),
        "notifications/initialized" => None,
        "ping" => id.map(|id| response(id, json!({}))),
        "tools/list" => id.map(|id| {
            response(
                id,
                json!({
                    "tools": tools.iter().map(tool_json).collect::<Vec<_>>()
                }),
            )
        }),
        "tools/call" => {
            let id = id?;
            Some(handle_tool_call(id, message.get("params").unwrap_or(&Value::Null), tools).await)
        }
        _ => id.map(|id| error_response(id, -32601, "Method not found")),
    }
}

async fn handle_tool_call(id: Value, params: &Value, tools: &[GeneratedTool]) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");

    let Some(tool) = tools.iter().find(|tool| tool.name == name) else {
        return error_response(id, -32602, "Unknown tool");
    };
    let empty_arguments = Value::Object(Default::default());
    let arguments = params.get("arguments").unwrap_or(&empty_arguments);

    match execute_tool(tool, arguments).await {
        Ok(output) => {
            let text = if output.success {
                output.stdout
            } else if !output.stderr.is_empty() {
                output.stderr
            } else {
                output.stdout
            };
            response(
                id,
                json!({
                    "content": [{
                        "type": "text",
                        "text": text
                    }],
                    "isError": !output.success
                }),
            )
        }
        Err(err) => response(
            id,
            json!({
                "content": [{
                    "type": "text",
                    "text": err.to_string()
                }],
                "isError": true
            }),
        ),
    }
}

fn tool_json(tool: &GeneratedTool) -> Value {
    json!({
        "name": tool.name,
        "title": tool.title,
        "description": tool.description,
        "inputSchema": tool.input_schema,
        "annotations": tool.annotations
    })
}

fn response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

pub async fn run_stdio() -> Result<()> {
    let tools = generated_tools();
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut lines = BufReader::new(stdin).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(&line) {
            Ok(message) => handle_message(message, &tools).await,
            Err(_) => Some(error_response(Value::Null, -32700, "Parse error")),
        };

        if let Some(response) = response {
            stdout
                .write_all(serde_json::to_string(&response)?.as_bytes())
                .await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn initialize_response_shape() {
        let response = handle_message(
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "test", "version": "0.0.0" }
                }
            }),
            &generated_tools(),
        )
        .await
        .unwrap();

        assert_eq!(response["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(
            response["result"]["capabilities"]["tools"]["listChanged"],
            false
        );
        assert_eq!(response["result"]["serverInfo"]["name"], "nap-mcp-server");
    }

    #[tokio::test]
    async fn tools_list_includes_generated_tools() {
        let response = handle_message(
            json!({ "jsonrpc": "2.0", "id": "tools", "method": "tools/list" }),
            &generated_tools(),
        )
        .await
        .unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();

        assert!(tools.iter().any(|tool| tool["name"] == "nap_resolve"));
        assert!(tools.iter().any(|tool| tool["name"] == "nap_remote_add"));
        assert!(tools.iter().any(|tool| tool["name"] == "nap_choose"));
    }

    #[test]
    fn output_format_is_forced_but_asset_format_is_not() {
        let tools = generated_tools();
        let resolve = tools
            .iter()
            .find(|tool| tool.name == "nap_resolve")
            .unwrap();
        let resolve_args =
            build_cli_args(resolve, &json!({ "uri": "nap://repo/character/id" })).unwrap();
        assert!(
            resolve_args
                .windows(2)
                .any(|pair| pair == ["--format", "json"])
        );

        let add_repr = tools.iter().find(|tool| tool.name == "nap_add").unwrap();
        let add_repr_args = build_cli_args(
            add_repr,
            &json!({
                "uri": "nap://repo/character/id",
                "key": "reference_image",
                "file": "image.png",
                "format": "png"
            }),
        )
        .unwrap();
        assert!(
            add_repr_args
                .windows(2)
                .any(|pair| pair == ["--format", "png"])
        );
        assert!(
            !add_repr_args
                .windows(2)
                .any(|pair| pair == ["--format", "json"])
        );
    }

    #[test]
    fn unknown_arguments_are_rejected() {
        let tool = generated_tools()
            .into_iter()
            .find(|tool| tool.name == "nap_status")
            .unwrap();
        let err = build_cli_args(&tool, &json!({ "extra": "nope" })).unwrap_err();
        assert!(err.to_string().contains("unknown argument"));
    }

    #[test]
    fn type_mismatches_are_rejected() {
        let tool = generated_tools()
            .into_iter()
            .find(|tool| tool.name == "nap_history")
            .unwrap();
        let err = build_cli_args(
            &tool,
            &json!({ "uri": "nap://repo/character/id", "limit": "20" }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("must be an integer"));
    }

    #[test]
    fn doctor_is_not_advertised_read_only() {
        let tool = generated_tools()
            .into_iter()
            .find(|tool| tool.name == "nap_doctor")
            .unwrap();
        assert_eq!(tool.annotations["readOnlyHint"], false);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn tool_call_invokes_fake_nap_with_args_and_env() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let log = temp.path().join("call.log");
        let fake_nap = temp.path().join("nap");
        fs::write(
            &fake_nap,
            format!(
                "#!/usr/bin/env bash\nprintf '%s\\n' \"$NAP_OUTPUT\" > {log}\nprintf '%s\\n' \"$@\" >> {log}\necho '{{\"ok\":true}}'\n",
                log = log.display()
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&fake_nap).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_nap, perms).unwrap();

        let tools = generated_tools();
        let resolve = tools
            .iter()
            .find(|tool| tool.name == "nap_resolve")
            .unwrap();
        let output = execute_tool_with_binary(
            resolve,
            &json!({ "uri": "nap://repo/character/id" }),
            fake_nap.to_str().unwrap(),
        )
        .await
        .unwrap();

        assert!(output.success);
        assert_eq!(output.stdout.trim(), "{\"ok\":true}");
        let log = fs::read_to_string(log).unwrap();
        assert!(log.lines().any(|line| line == "json"));
        assert!(log.lines().any(|line| line == "resolve"));
        assert!(log.lines().any(|line| line == "nap://repo/character/id"));
        assert!(log.lines().any(|line| line == "--format"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cli_errors_become_tool_errors() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let fake_nap = temp.path().join("nap");
        fs::write(&fake_nap, "#!/usr/bin/env bash\necho nope >&2\nexit 7\n").unwrap();
        let mut perms = fs::metadata(&fake_nap).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_nap, perms).unwrap();

        let mut tool = generated_tools()
            .into_iter()
            .find(|tool| tool.name == "nap_status")
            .unwrap();
        tool.command = vec![fake_nap.to_string_lossy().to_string()];
        tool.params = vec![];

        let output = execute_tool_with_binary(&tool, &json!({}), fake_nap.to_str().unwrap())
            .await
            .unwrap();
        assert!(!output.success);
        assert!(output.stderr.contains("nope"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn timeout_kills_child_process() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("still-running");
        let fake_nap = temp.path().join("nap");
        fs::write(
            &fake_nap,
            format!(
                "#!/usr/bin/env bash\ntrap 'exit 0' TERM\nsleep 5\ntouch {marker}\n",
                marker = marker.display()
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&fake_nap).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_nap, perms).unwrap();

        let mut tool = generated_tools()
            .into_iter()
            .find(|tool| tool.name == "nap_status")
            .unwrap();
        tool.timeout_ms = 50;

        let err = execute_tool_with_binary(&tool, &json!({}), fake_nap.to_str().unwrap())
            .await
            .unwrap_err();
        assert!(err.to_string().contains("timed out"));
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!marker.exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn output_larger_than_limit_is_rejected() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let fake_nap = temp.path().join("nap");
        fs::write(
            &fake_nap,
            "#!/usr/bin/env bash\npython3 - <<'PY'\nprint('x' * (10 * 1024 * 1024 + 1))\nPY\n",
        )
        .unwrap();
        let mut perms = fs::metadata(&fake_nap).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_nap, perms).unwrap();

        let tool = generated_tools()
            .into_iter()
            .find(|tool| tool.name == "nap_status")
            .unwrap();
        let err = execute_tool_with_binary(&tool, &json!({}), fake_nap.to_str().unwrap())
            .await
            .unwrap_err();
        assert!(
            err.chain()
                .any(|cause| cause.to_string().contains("output exceeded"))
        );
    }
}

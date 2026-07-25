use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "nap-mcp-server {}\n\nUsage:\n  nap-mcp-server\n\nStarts a stdio MCP server that proxies tool calls to the host nap CLI.\nThe process is intended to be launched on demand by an MCP client.",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }

    nap_mcp_server::run_stdio().await
}

mod cli;
mod core;
mod mcp;
mod storage;

fn main() -> anyhow::Result<()> {
    cli::run()
}

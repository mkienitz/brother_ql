mod app;
mod cli;
mod connection;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{EnvFilter, field::MakeExt};

use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new(if cli.debug { "debug" } else { "info" }))
        .init();

    app::run(cli.command)
}

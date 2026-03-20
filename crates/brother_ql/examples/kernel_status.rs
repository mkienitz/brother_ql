//! Read and display printer status via a kernel driver connection
use std::{env, error::Error};

use brother_ql::connection::{KernelConnection, PrinterConnection};
use tracing_subscriber::{EnvFilter, field::MakeExt};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();

    // Parse printer device path or default to the most likely one
    let path = env::args().nth(1).unwrap_or_else(|| "/dev/usb/lp0".into());

    // Open kernel connection
    let mut connection = KernelConnection::open(&path)?;
    // Read status from printer
    let status = connection.get_status()?;
    print!("{status}");

    Ok(())
}

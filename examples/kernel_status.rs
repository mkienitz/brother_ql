//! Read and display printer status via a kernel driver connection
use std::{env, error::Error};

use brother_ql::connection::{KernelConnection, PrinterConnection};
use tracing_subscriber::{field::MakeExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Use the most common device path but allow others
    let path = env::args().nth(1).unwrap_or_else(|| "/dev/usb/lp0".into());
    // Open kernel connection
    let mut connection = KernelConnection::open(&path)?;
    // Read status from printer
    let _status = connection.get_status()?;
    Ok(())
}

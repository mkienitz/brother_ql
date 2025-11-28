//! Read and display printer status via a kernel driver connection
use std::error::Error;

use brother_ql::connection::{KernelConnection, PrinterConnection};
use tracing_subscriber::{EnvFilter, field::MakeExt};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Open kernel connection
    let mut connection = KernelConnection::open("/dev/usb/lp0")?;
    // Read status from printer
    let _status = connection.get_status()?;
    Ok(())
}

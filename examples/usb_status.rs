//! Read and display printer status via USB connection

use std::error::Error;

use brother_ql::connection::{PrinterConnection, UsbConnection, UsbConnectionInfo};
use tracing_subscriber::{field::MakeExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Create connection info for whatever printer is connected
    let info = UsbConnectionInfo::discover()?.expect("No supported printer found");
    // Open USB connection
    let mut connection = UsbConnection::open(info)?;
    // Read status from printer
    let _status = connection.get_status()?;
    Ok(())
}

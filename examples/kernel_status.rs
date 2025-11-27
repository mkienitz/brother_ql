//! Read and display printer status via kernel interface

use std::error::Error;

use brother_ql::connection::KernelConnection;
use tracing_subscriber::{field::MakeExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses logging
    tracing_subscriber::fmt()
        .map_fmt_fields(|f| f.debug_alt())
        .with_env_filter(EnvFilter::new("debug"))
        .init();

    // Open kernel connection
    let mut connection = KernelConnection::open("/dev/usb/lp0")?;
    // Read status from printer
    let status = connection.get_status()?;
    println!("Model: {:?}", status.model);

    Ok(())
}

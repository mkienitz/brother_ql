//! Print 62mm continuous labels via USB connection

use std::error::Error;

use brother_ql::{
    connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
    media::Media,
    printer::PrinterModel,
    printjob::PrintJob,
};
use tracing_subscriber::{field::MakeExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Create connection info for QL-820NWB
    let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    // Open USB connection
    let mut connection = UsbConnection::open(info)?;
    // Read status from printer
    let _status = connection.get_status()?;
    // Create a print job with more than one page
    let img = image::open("c62.png")?;
    let job = PrintJob::new(img, Media::C62)?.page_count(2);
    // These are the defaults for the other options:
    // .high_dpi(false)
    // .compressed(false)
    // .quality_priority(true)
    // .cut_behavior(CutBehavior::CutEach)?; // default for continuous media
    // Finally, print
    connection.print(job)?;
    Ok(())
}

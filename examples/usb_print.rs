use std::error::Error;

use brother_ql::{
    connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
    media::Media,
    printer::PrinterModel,
    printjob::{CutBehavior, PrintJob},
};
use tracing_subscriber::{EnvFilter, field::MakeExt};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(|f| f.debug_alt())
        .with_env_filter(EnvFilter::new("debug"))
        .init();

    // Create connection info for QL-820NWB
    let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    // Open USB connection
    let mut connection = UsbConnection::open(info)?;
    // Read status from printer
    let _status = connection.get_status()?;
    // Create a print job with more than one page
    let img = image::open("test.png")?;
    let job = PrintJob {
        no_pages: 2,
        image: img,
        media: Media::C62,
        high_dpi: false,
        compressed: false,
        quality_priority: true,
        cut_behavior: CutBehavior::CutEach,
    };
    // Finally, print
    connection.print(job)?;

    Ok(())
}

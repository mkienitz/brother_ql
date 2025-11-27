use std::error::Error;

use brother_ql::{
    connection::{KernelConnection, PrinterConnection},
    media::Media,
    printjob::PrintJob,
};
use tracing_subscriber::{field::MakeExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(|f| f.debug_alt())
        .with_env_filter(EnvFilter::new("debug"))
        .init();

    // Open kernel connection
    let mut connection = KernelConnection::open("/dev/usb/lp0")?;
    // Read status from printer
    let _status = connection.get_status()?;
    // Create a print job with more than one page
    let img = image::open("test.png")?;
    let job = PrintJob::new(img, Media::C62)?;
    // These are the defaults for the other options:
    // .high_dpi(false)
    // .compressed(false)
    // .quality_priority(true)
    // .cut_behavior(CutBehavior::CutEach)?; // default for continuous media
    // Finally, print
    connection.print(job)?;

    Ok(())
}

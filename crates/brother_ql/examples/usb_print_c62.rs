//! Print 62mm continuous labels via USB connection

use std::error::Error;

use brother_ql::{
    connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
    media::Media,
    printjob::PrintJobBuilder,
    test_labels,
};
use tracing_subscriber::{EnvFilter, field::MakeExt};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Create connection info for whatever printer is connected
    let info = UsbConnectionInfo::discover()?.expect("No supported printer found");
    // Open USB connection
    let mut connection = UsbConnection::open(info)?;
    // Use a test label for demo purposes (generated using typst)
    let img = test_labels::render_test_label(Media::C62)?;
    // Create a print job with more than one page
    let job = PrintJobBuilder::new(Media::C62)
        .add_label(img)
        .copies(1)
        // These are the defaults for the other options:
        // .high_dpi(false)
        // .compressed(false)
        // .quality_priority(true)
        // .cut_behavior(CutBehavior::CutEach) // default for continuous media like C62
        .build()?;
    // Finally, print
    connection.print(job)?;
    Ok(())
}

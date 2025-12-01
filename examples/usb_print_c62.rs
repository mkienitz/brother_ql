//! Print 62mm continuous labels via USB connection

use std::{env, error::Error};

use brother_ql::{
    connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
    media::Media,
    printjob::PrintJobBuilder,
};
use tracing_subscriber::{field::MakeExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Take an image path from the command line
    let mut args = env::args();
    let prog = args.next().unwrap();
    let Some(img_path) = args.next() else {
        println!("Usage: {prog} <image>");
        return Ok(());
    };
    // Create connection info for whatever printer is connected
    let info = UsbConnectionInfo::discover()?.expect("No supported printer found");
    // Open USB connection
    let mut connection = UsbConnection::open(info)?;
    // Read status from printer
    let _status = connection.get_status()?;
    // Create a print job with more than one page
    let img = image::open(img_path)?;
    let job = PrintJobBuilder::new(Media::C62)
        .add_label(img)
        // These are the defaults for the other options:
        // .copies(1)
        // .high_dpi(false)
        // .compressed(false)
        // .quality_priority(true)
        // .cut_behavior(CutBehavior::CutEach) // default for continuous media like C62
        .build()?;
    // Finally, print
    connection.print(job)?;
    Ok(())
}

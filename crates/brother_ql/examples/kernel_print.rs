//! Print 62mm continuous labels via kernel driver connection
use std::{env, error::Error};

use brother_ql::{
    connection::{KernelConnection, PrinterConnection},
    media::Media,
    printjob::PrintJobBuilder,
};
use tracing_subscriber::{EnvFilter, field::MakeExt};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();

    // Accept an image path as a CLI argument
    let mut args = env::args();
    let prog = args.next().unwrap();
    let Some(img_path) = args.next() else {
        println!("Usage: {prog} <image> [device]");
        return Ok(());
    };
    // Default to the most likely printer device path
    let device_path = args.next().unwrap_or_else(|| "/dev/usb/lp0".into());

    // Create a print job
    let img = image::open(img_path)?;
    let job = PrintJobBuilder::new(Media::C62)
        .add_label(img)
        // These are the defaults for the other options:
        // .copies(NonZeroU8::MIN)
        // .high_dpi(false)
        // .compressed(false)
        // .quality_priority(true)
        // .cut_behavior(CutBehavior::CutEach) // default for continuous media like C62
        .build()?;

    // Open a kernel connection and print
    let mut connection = KernelConnection::open(device_path)?;
    connection.print(job)?;

    Ok(())
}

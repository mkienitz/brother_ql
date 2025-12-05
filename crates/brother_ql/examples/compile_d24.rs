//! Compile a print job for 24mm die-cut labels and save to file

use std::{error::Error, fs::File, io::Write};

use brother_ql::{media::Media, printjob::PrintJob};
use tracing_subscriber::{EnvFilter, field::MakeExt};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Make sure to use a compatible image (236x236 pixels wide)
    let img = image::open("d24.png")?;
    // Compile and save print job
    let job = PrintJob::from_image(img, Media::D24)?;
    let data = job.compile();
    let mut file = File::create("d24.bin")?;
    file.write_all(&data)?;
    Ok(())
}

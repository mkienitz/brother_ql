#![allow(missing_docs)]

use std::{error::Error, fs::File, io::Write};

use brother_ql::{media::Media, printjob::PrintJob};
use tracing_subscriber::{EnvFilter, field::MakeExt};

pub fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Make sure to use a compatible image (696px wide)
    let img = image::open("c62.png")?;
    let job = PrintJob::new(img, Media::C62)?;
    // These are the defaults for the other options:
    // .page_count(1)
    // .high_dpi(false)
    // .compressed(false)
    // .quality_priority(true)
    // .cut_behavior(CutBehavior::CutEach)?; // default for continuous media
    let data = job.compile();
    let mut file = File::create("c62mm.bin")?;
    let _ = file.write(&data);
    Ok(())
}

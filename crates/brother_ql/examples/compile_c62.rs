//! Compile a print job for 62mm continuous media and save to file

use std::{error::Error, fs::File, io::Write};

use brother_ql::{media::Media, printjob::PrintJobBuilder};
use tracing_subscriber::{EnvFilter, field::MakeExt};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    // Make sure to use a compatible image (696px wide)
    let img = image::open("c62.png")?;
    let job = PrintJobBuilder::new(Media::C62).add_label(img).build()?;
    let data = job.compile();
    let mut file = File::create("c62mm.bin")?;
    file.write_all(&data)?;
    Ok(())
}

//! Compile a print job and save to file

use std::{env, error::Error, fs::File, io::Write};

use brother_ql::{media::Media, printjob::PrintJobBuilder};
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
        println!("Usage: {prog} <image>");
        return Ok(());
    };

    // NOTE: Make sure to use an image compatible with the chosen media
    let img = image::open(&img_path)?;
    let job = PrintJobBuilder::new(Media::C62).add_label(img).build()?;
    let data = job.compile();
    let mut file = File::create("job.bin")?;
    file.write_all(&data)?;

    // You could then send this compiled print job directly to a printer, e.g. via netcat:
    // $ nc <IP> 9100 < job.bin

    Ok(())
}

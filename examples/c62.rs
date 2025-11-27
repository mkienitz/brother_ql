use std::{error::Error, fs::File, io::Write};

use brother_ql::{media::Media, printjob::PrintJob};

pub fn main() -> Result<(), Box<dyn Error>> {
    // Make sure to use a compatible image (696px wide)
    let img = image::open("c62.png")?;
    let job = PrintJob::new(img, Media::C62)?;
    // These are the defaults for the other options:
    // .page_count(1)
    // .high_dpi(false)
    // .compressed(false)
    // .quality_priority(true)
    // .cut_behavior(CutBehavior::CutEach)?; // default for continuous media
    let data = job.compile()?;
    let mut file = File::create("c62mm.bin")?;
    let _ = file.write(&data);
    Ok(())
}

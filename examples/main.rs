use std::{error::Error, fs::File, io::Write};

use brother_ql::{media::Media, job::{PrintJob, CutBehavior}};


pub fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open("test.png")?;
    let job = PrintJob {
        no_pages: 1,
        image: img,
        media: Media::C62,
        high_dpi: false,
        compressed: false,
        quality_priority: true,
        cut_behaviour: CutBehavior::CutAtEnd,
    };
    let data = job.render()?;

    let mut file = File::create("test.bin")?;
    let _ = file.write(&data);
    Ok(())
}

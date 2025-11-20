use std::{error::Error, fs::File, io::Write};

use brother_ql::{
    media::Media,
    printjob::{CutBehavior, PrintJob},
};

pub fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open("test.png")?;
    let job = PrintJob {
        no_pages: 1,
        image: img,
        media: Media::C62,
        high_dpi: false,
        compressed: false,
        quality_priority: true,
        cut_behavior: CutBehavior::CutEach,
    };
    let data = job.compile()?;
    let mut file = File::create("test.bin")?;
    let _ = file.write(&data);
    Ok(())
}

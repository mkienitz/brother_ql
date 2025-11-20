//! This is a crate to convert image data to the Raster Command binary data understood by the
//! Brother QL-820NWB label printer.
//!
//! * It is still very much work-in-progress so some bugs might still exist.
//! * Currently, only the 820NWB printer is supported but other printers should be relatively easy to
//!   add - especially the 8xx sibling models.
//! * The two-color (red and black) printing mode is supported
//! * The image is represented by [DynamicImage][image::DynamicImage] from the [image] crate
//! * For details, check the [official Raster Command Reference](https://download.brother.com/welcome/docp100278/cv_ql800_eng_raster_101.pdf)
//!
//! Here is a small example on how to use it:
//!
//! ```no_run
//! use std::{error::Error, fs::File, io::Write};
//!
//! use brother_ql::{
//!     printjob::{CutBehavior, PrintJob},
//!     media::Media,
//! };
//!
//! pub fn main() -> Result<(), Box<dyn Error>> {
//!     let img = image::open("test.png")?;
//!     let job = PrintJob {
//!         no_pages: 1,
//!         image: img,
//!         media: Media::C62,       // use 62mm wide continuous tape
//!         high_dpi: false,
//!         compressed: false,       // unsupported
//!         quality_priority: false, // no effect on two-color printing
//!         cut_behavior: CutBehavior::CutAtEnd,
//!     };
//!     let data = job.compile()?;
//!     let mut file = File::create("test.bin")?;
//!     let _ = file.write(&data);
//!     // We can now send this binary directly to the printer, for example using `nc`
//!     Ok(())
//! }
//!
//! ```
#![warn(missing_docs)]
mod commands;
pub mod error;
pub mod media;
pub mod printjob;
mod raster_image;
mod status;

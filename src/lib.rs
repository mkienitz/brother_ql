//! This is a crate to convert image data to the Raster Command binary data understood by the
//! Brother QL-820NWB label printer.
//!
//! * It is still very much work-in-progress so some bugs might still exist.
//! * Currently, only the 820NWB printer is supported but other printers should be relatively easy to
//!   add - especially the 8xx sibling models.
//! * The two-color (red and black) printing mode is supported
//! * The image is represented by [`DynamicImage`][image::DynamicImage] from the [image] crate
//! * For details, check the [official Raster Command Reference](https://download.brother.com/welcome/docp100278/cv_ql800_eng_raster_101.pdf)
//!
//! Here is a small example on how to use it:
//!
//! ```no_run
//! use std::error::Error;
//!
//! use brother_ql::{
//!     connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
//!     media::Media,
//!     printer::PrinterModel,
//!     printjob::PrintJob,
//! };
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     // Create connection info for QL-820NWB
//!     let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
//!     // Open USB connection
//!     let mut connection = UsbConnection::open(info)?;
//!     // Read status from printer
//!     let _status = connection.get_status()?;
//!     // Create a print job with more than one page
//!     let img = image::open("test.png")?;
//!     let job = PrintJob::new(&img, Media::C62)?.page_count(2);
//!     // These are the defaults for the other options:
//!     // .high_dpi(false)
//!     // .compressed(false)
//!     // .quality_priority(true)
//!     // .cut_behavior(CutBehavior::CutEach); // default for continuous media
//!     // Finally, print
//!     connection.print(job)?;
//!
//!     Ok(())
//! }
//! ```

mod commands;
#[cfg(feature = "usb")]
pub mod connection;
pub mod error;
pub mod media;
pub mod printer;
pub mod printjob;
mod raster_image;
#[cfg(feature = "usb")]
pub mod status;

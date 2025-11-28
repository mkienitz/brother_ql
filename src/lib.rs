//! This is a crate to convert image data to the Raster Command binary data understood by
//! Brother QL series label printers.
//!
//! * It is still very much work-in-progress so some bugs might still exist.
//! * Many Brother QL models are supported (5xx, 6xx, 7xx, 8xx series), and other printers should be relatively easy to add
//! * The two-color (red and black) printing mode is supported when using a printer with this
//!   capability
//! * The image is represented by [`DynamicImage`][image::DynamicImage] from the [image] crate
//! * For more details, check the [official Raster Command Reference](https://download.brother.com/welcome/docp100278/cv_ql800_eng_raster_101.pdf)
//!
//! # Example: Printing via an USB connection
//!
//! ```no_run
//! use brother_ql::{
//!     connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
//!     media::Media, printer::PrinterModel, printjob::PrintJob,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create connection info for QL-820NWB
//! let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
//! // Open USB connection
//! let mut connection = UsbConnection::open(info)?;
//! // Read status from printer
//! let _status = connection.get_status()?;
//! // Create a print job with more than one page
//! let img = image::open("c62.png")?;
//! let job = PrintJob::new(img, Media::C62)?.page_count(2);
//! // These are the defaults for the other options:
//! // .high_dpi(false)
//! // .compressed(false)
//! // .quality_priority(true)
//! // .cut_behavior(CutBehavior::CutEach)?; // default for continuous media
//! // Finally, print
//! connection.print(job)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Compiling and saving a print job
//!
//! ```no_run
//! use brother_ql::{
//!     media::Media, printer::PrinterModel, printjob::PrintJob,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let img = image::open("c62.png")?;
//! let job = PrintJob::new(img, Media::C62)?;
//! let data = job.compile();
//! let mut file = File::create("c62mm.bin")?;
//! file.write_all(&data)?;
//! # Ok(())
//! # }
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

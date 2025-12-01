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
//!     media::Media,
//!     printer::PrinterModel,
//!     printjob::PrintJob,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to a specific printer model
//! let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
//! let mut connection = UsbConnection::open(info)?;
//!
//! // Create and print a label
//! let img = image::open("label.png")?;
//! let job = PrintJob::from_image(img, Media::C62)?;
//! connection.print(job)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Kernel Driver Printing (Linux, no features required)
//!
//! ```no_run
//! use brother_ql::{
//!     connection::{KernelConnection, PrinterConnection},
//!     media::Media,
//!     printjob::PrintJob,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut connection = KernelConnection::open("/dev/usb/lp0")?;
//! let img = image::open("label.png")?;
//! let job = PrintJob::from_image(img, Media::C62)?;
//! connection.print(job)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Compile to File (no features required)
//!
//! ```no_run
//! use brother_ql::{media::Media, printjob::PrintJob};
//! use std::{fs::File, io::Write};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let img = image::open("label.png")?;
//! let job = PrintJob::from_image(img, Media::C62)?;
//!
//! // Compile to binary data
//! let data = job.compile();
//!
//! // Save to file (can be sent via network: `nc printer-ip 9100 < output.bin`)
//! File::create("output.bin")?.write_all(&data)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Choosing Media
//!
//! Media types determine label dimensions and behavior:
//!
//! - **Continuous rolls** (`C` prefix like `C62`) - Cut to any length
//! - **Die-cut labels** (`D` prefix like `D17x54`) - Pre-cut, fixed dimensions
//! - **Two-color** (`R` suffix like `C62R`) - Black/red printing support
//!
//! All media types require **720 pixels wide** images at 300 DPI. Height varies by media type.
//! See [`media`] module for complete details.
//!
//! # Print Job Configuration
//!
//! Use [`PrintJobBuilder`](printjob::PrintJobBuilder) for advanced customization:
//!
//! ```no_run
//! # use brother_ql::{media::Media, printjob::{PrintJobBuilder, CutBehavior}};
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let img1 = image::open("label1.png")?;
//! # let img2 = image::open("label2.png")?;
//! let job = PrintJobBuilder::new(Media::C62)
//!     .add_label(img1)                    // Add first image
//!     .add_label(img2)                    // Add second image
//!     .copies(5)                          // Print 5 copies of each
//!     .high_dpi(false)                    // 300 DPI (default)
//!     .quality_priority(true)             // Quality over speed (default)
//!     .cut_behavior(CutBehavior::CutEvery(2))  // Cut every 2 labels
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! See [`PrintJob`] and [`PrintJobBuilder`](printjob::PrintJobBuilder) for defaults and all options.

mod commands;
pub mod connection;
pub mod error;
pub mod media;
pub mod printer;
pub mod printjob;
mod raster_image;
pub mod status;

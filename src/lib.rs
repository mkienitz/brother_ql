//! Rust driver for Brother QL series label printers
//!
//! Convert images to Brother QL Raster Command binary data and print labels via USB or kernel drivers.
//!
//! # Features
//!
//! - **Print directly** via USB ([`UsbConnection`](connection::UsbConnection)) or Linux kernel driver ([`KernelConnection`](connection::KernelConnection))
//! - **Compile to file** for network printing or debugging
//! - **Multiple label types** - Continuous rolls and die-cut labels in various sizes
//! - **Two-color printing** - Black/red support on compatible printers
//! - **Full status monitoring** - Check errors, media type, and printer state
//!
//! # Supported Printers
//!
//! Brother QL series: **5xx** (QL-560, QL-570, QL-580N), **6xx** (QL-600, QL-650TD),
//! **7xx** (QL-700, QL-710W, QL-720NW), **8xx** (QL-800, QL-810W, QL-820NWB)
//!
//! See [`printer`] module for complete list and testing status.
//!
//! # Feature Flags
//!
//! - **`usb`** (optional) - Enables USB printing via the `rusb` crate. Provides [`UsbConnection`](connection::UsbConnection)
//!   and [`UsbConnectionInfo`](connection::UsbConnectionInfo).
//! - **`serde`** (optional) - Enables serialization support for [`Media`] and [`CutBehavior`](printjob::CutBehavior).
//!
//! The crate has **no default features**. Basic print job compilation and [`KernelConnection`](connection::KernelConnection)
//! work without any features enabled.
//!
//! # Quick Start
//!
//! ## USB Printing (requires `usb` feature)
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
//!
//! # Status Monitoring
//!
//! Check printer status, errors, and media information:
//!
//! ```no_run
//! # use brother_ql::connection::{PrinterConnection, UsbConnection, UsbConnectionInfo};
//! # use brother_ql::printer::PrinterModel;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
//! # let mut connection = UsbConnection::open(info)?;
//! let status = connection.get_status()?;
//!
//! println!("Model: {:?}", status.model);
//! println!("Media: {}mm wide", status.media_width);
//!
//! if status.has_errors() {
//!     eprintln!("Errors: {:?}", status.errors);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! See [`status`] module for complete status information.
//!
//! # References
//!
//! - [Official Raster Command Reference](https://download.brother.com/welcome/docp100278/cv_ql800_eng_raster_101.pdf)
//! - Images are processed using the [`image`] crate
//!
//! [`Media`]: media::Media
//! [`PrintJob`]: printjob::PrintJob

mod commands;
pub mod connection;
pub mod error;
pub mod media;
pub mod printer;
pub mod printjob;
mod raster_image;
pub mod status;

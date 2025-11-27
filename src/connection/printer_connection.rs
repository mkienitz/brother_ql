//! Trait defining common printer connection behavior

use crate::{error::PrintError, printjob::PrintJob};

/// Common interface for all printer connections (USB, Network, etc.)
///
/// This trait provides a unified interface for sending print jobs to Brother QL printers,
/// regardless of the underlying connection type (USB, network, etc.).
///
/// # Example
/// ```no_run
/// # use brother_ql::{
/// #     connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
/// #     media::Media,
/// #     printer::PrinterModel,
/// #     printjob::PrintJob,
/// # };
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Open a connection (USB in this example)
/// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
/// let mut connection = UsbConnection::open(info)?;
///
/// // Create a print job
/// let image = image::open("label.png")?;
/// let job = PrintJob::new(&image, Media::C62)?;
///
/// // Print using the trait method
/// connection.print(job)?;
/// # Ok(())
/// # }
/// ```
pub trait PrinterConnection {
    /// Send a print job to the printer
    ///
    /// This method compiles the print job into raster commands and sends them
    /// to the printer, waiting for status confirmations at each stage.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Communication with the printer fails
    /// - The printer reports an error (paper jam, out of media, etc.)
    /// - The print job cannot be compiled
    /// - Status validation fails during printing
    fn print(&mut self, job: PrintJob) -> Result<(), PrintError>;
}

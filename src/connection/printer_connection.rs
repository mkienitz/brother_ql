//! Trait defining common printer connection behavior

use crate::{error::BQLError, printjob::PrintJob};

/// Common interface for all printer connections (USB, Network, etc.)
pub trait PrinterConnection {
    /// Print a job to the printer
    fn print(&mut self, job: PrintJob) -> Result<(), BQLError>;
}

//! Trait defining common printer connection behavior

use tracing::{debug, info};

use crate::{
    commands::RasterCommands,
    connection::printer_connection::sealed::ConnectionImpl,
    error::{PrintError, PrintErrorSource, StatusError},
    printjob::PrintJob,
    status::{Phase, StatusInformation, StatusType},
};

/// Sealed trait to prevent external implementations
pub(super) mod sealed {
    use std::time::Duration;

    use strum::IntoEnumIterator;
    use tracing::debug;

    use crate::{
        commands::RasterCommand,
        error::{ConnectionError, ProtocolError, StatusError},
        media::{LengthInfo, Media, MediaSettings},
        status::{Phase, StatusInformation, StatusType},
    };

    pub trait ConnectionImpl {
        type Error: std::error::Error + Send + Sync + 'static + ConnectionError;

        /// Write data to the printer
        ///
        /// # Errors
        /// Returns an error if the write operation fails or if not all data could be written.
        fn write(&mut self, data: &[u8]) -> Result<(), Self::Error>;

        /// Read data from the printer
        ///
        /// Returns the number of bytes read into the buffer.
        ///
        /// # Errors
        /// Returns an error if the read operation fails.
        fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;

        /// Read status information but without sending init/invalidate bytes
        ///
        /// # Errors
        /// Returns an error if:
        /// - Communication with the printer fails
        /// - The status reply is malformed or incomplete
        fn read_status_reply(&mut self) -> Result<StatusInformation, StatusError<Self::Error>> {
            let mut read_buffer = [0u8; 32];
            self.read_exact(&mut read_buffer)?;
            let status =
                StatusInformation::try_from(&read_buffer[..]).map_err(StatusError::Parsing)?;
            debug!(?status, "Printer sent status information");
            Ok(status)
        }

        /// Read until the provided buffer is full
        ///
        /// # Errors
        /// Returns an error if:
        /// - Communication with the printer fails
        /// - The printer does not respond within the timeout period
        fn read_exact(&mut self, buffer: &mut [u8]) -> Result<(), StatusError<Self::Error>> {
            // 3000ms / 50ms = 60 retries
            const MAX_RETRIES: u8 = 60;
            const RETRY_DELAY: Duration = Duration::from_millis(50);

            let mut total_read = 0;
            let mut retries = 0;

            while total_read < buffer.len() {
                match self.read(&mut buffer[total_read..]) {
                    Ok(0) => {
                        retries += 1;
                        if retries > MAX_RETRIES {
                            return Err(StatusError::NoResponse);
                        }
                        // No data available yet, wait and retry
                        std::thread::sleep(RETRY_DELAY);
                    }
                    Ok(n) => {
                        total_read += n;
                        retries = 0; // Reset retries on successful read
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Ok(())
        }

        /// Send a status information request to the printer
        ///
        /// # Errors
        /// Returns an error if the write operation fails
        fn send_status_request(&mut self) -> Result<(), Self::Error> {
            debug!("Sending status information request to the printer...");
            let status_request_bytes: Vec<u8> = RasterCommand::StatusInformationRequest.into();
            self.write(&status_request_bytes)?;
            Ok(())
        }

        /// Validate that information reply matches expected state
        ///
        /// # Errors
        /// Returns an error if:
        /// - The printer reports error conditions
        /// - The status type or phase doesn't match expectations
        fn validate_status(
            status: &StatusInformation,
            job_media: Media,
            expected_type: &StatusType,
            expected_phase: &Phase,
        ) -> Result<(), ProtocolError> {
            // Validate that the printer has the correct media installed
            fn status_matches_media(status: &StatusInformation, media: Media) -> bool {
                let media_settings = MediaSettings::from(media);
                let media_type_matches = status.media_type == Some(media_settings.media_type);
                let media_width_matches = status.media_width == media_settings.width_mm;
                let media_length_matches = match media_settings.length_info {
                    LengthInfo::Endless => status.media_length == 0,
                    LengthInfo::Fixed { length_mm, .. } => status.media_length == length_mm,
                };
                media_type_matches && media_width_matches && media_length_matches
            }
            if !status_matches_media(status, job_media) {
                // Find likely match for reported media
                let likely_match = Media::iter().find(|&m| status_matches_media(status, m));
                return Err(ProtocolError::MediaMismatch {
                    expected_media: job_media,
                    reported_media: likely_match,
                });
            }

            // Check if printer has errors first
            if status.has_errors() {
                return Err(ProtocolError::PrinterError(status.errors));
            }

            // Check if status type and phase match expectations
            if &status.status_type != expected_type || &status.phase != expected_phase {
                return Err(ProtocolError::UnexpectedStatus {
                    expected_type: expected_type.clone(),
                    expected_phase: expected_phase.clone(),
                    actual_type: status.status_type.clone(),
                    actual_phase: status.phase.clone(),
                });
            }

            Ok(())
        }
    }
}

/// Common interface for all printer connections (USB, Network, etc.)
///
/// This trait provides a unified interface for sending print jobs to Brother QL printers,
/// regardless of the underlying connection type (USB, network, etc.).
///
/// # Available Methods
///
/// - [`print`](PrinterConnection::print) - Send a print job to the printer
/// - [`get_status`](PrinterConnection::get_status) - Read detailed printer status
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
/// let job = PrintJob::new(image, Media::C62)?;
///
/// // Print using the trait method
/// connection.print(job)?;
///
/// // Check printer status
/// let status = connection.get_status()?;
/// println!("Printer: {:?}, Media: {}mm", status.model, status.media_width);
/// # Ok(())
/// # }
/// ```
pub trait PrinterConnection: ConnectionImpl {
    /// Send a print job to the printer
    ///
    /// This method compiles the print job into raster commands and sends them
    /// to the printer, waiting for status confirmations at each stage.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Communication with the printer fails (connection-type specific)
    /// - The printer reports an error (paper jam, out of media, etc.) or an unexpected state
    /// - Status information sent by the printer fails during printing
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
    /// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    /// let mut connection = UsbConnection::open(info)?;
    ///
    /// let image = image::open("label.png")?;
    /// let job = PrintJob::new(image, Media::C62)?;
    ///
    /// connection.print(job)?;
    /// # Ok(())
    /// # }
    /// ```
    fn print(&mut self, job: PrintJob) -> Result<(), PrintError<Self::Error>> {
        info!(?job, "Starting print job...");
        let no_pages = job.page_count;
        let expected_media = job.media;
        let parts = job.into_parts();
        // Send preamble
        self.write(&parts.preamble.build())
            // TODO: Decide on error mapping API
            // .map_err(|e| PrintError::with_page(e, 0))?;
            .map_err(PrintError::err_source_mapper(0))?;
        // Send status information request and validate printer is ready
        let status = self
            .get_status()
            .map_err(PrintError::err_source_mapper(0))?;
        Self::validate_status(
            &status,
            expected_media,
            &StatusType::StatusRequestReply,
            &Phase::Receiving,
        )
        .map_err(|e| PrintError::with_page(e, 0))?;

        for (page_no, page) in parts.page_data.into_iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let current_page = (page_no + 1) as u32;
            // We use a closure since try blocks are not available yet
            let page_res: Result<(), PrintErrorSource<Self::Error>> = (|| {
                self.write(&page.build())?;
                // Printer should change phase to "Printing"
                let status = self.read_status_reply()?;
                Self::validate_status(
                    &status,
                    expected_media,
                    &StatusType::PhaseChange,
                    &Phase::Printing,
                )?;
                // Printer should signal print completion
                let status = self.read_status_reply()?;
                Self::validate_status(
                    &status,
                    expected_media,
                    &StatusType::PrintingCompleted,
                    &Phase::Printing,
                )?;
                // Printer should change phase to "Receiving" again
                let status = self.read_status_reply()?;
                Self::validate_status(
                    &status,
                    expected_media,
                    &StatusType::PhaseChange,
                    &Phase::Receiving,
                )?;
                Ok(())
            })();
            page_res.map_err(PrintError::err_source_mapper(current_page))?;
            debug!(
                "Sending print data for page {}/{}...",
                current_page, no_pages
            );
            info!("Page {}/{} printed successfully!", current_page, no_pages);
        }
        info!("Print job completed successfully!");
        Ok(())
    }

    /// Read status information from the printer
    ///
    /// Sends a status request to the printer and returns detailed [`StatusInformation`] about:
    /// - Printer model
    /// - Current errors (if any)
    /// - Media type and dimensions
    /// - Current phase (receiving, printing, etc.)
    /// - Various mode settings
    ///
    /// This method sends the initialization preamble before requesting status.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Communication with the printer fails
    /// - The status reply is malformed or incomplete
    /// - Timeout occurs while waiting for response
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::{
    /// #     connection::{UsbConnection, UsbConnectionInfo},
    /// #     printer::PrinterModel,
    /// # };
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    /// let mut connection = UsbConnection::open(info)?;
    ///
    /// let status = connection.get_status()?;
    /// println!("Printer: {:?}", status.model);
    /// println!("Media width: {}mm", status.media_width);
    ///
    /// if status.has_errors() {
    ///     eprintln!("Printer errors: {:?}", status.errors);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn get_status(&mut self) -> Result<StatusInformation, StatusError<Self::Error>> {
        let preamble_bytes = RasterCommands::create_preamble().build();
        self.write(&preamble_bytes)?;
        self.send_status_request()?;
        self.read_status_reply()
    }
}

//! USB connection support for Brother QL printers
use std::time::Duration;

use rusb::{Context, Device, DeviceHandle, UsbContext};
use tracing::debug;

use crate::{error::UsbError, printer::PrinterModel};

use super::{PrinterConnection, printer_connection::sealed::ConnectionImpl};

/// USB connection parameters for a Brother QL printer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbConnectionInfo {
    /// USB vendor ID (typically 0x04f9 for Brother Industries, Ltd)
    pub(crate) vendor_id: u16,
    /// USB product ID (specific to each printer model)
    pub(crate) product_id: u16,
    /// USB interface number (typically 0 for printers)
    pub(crate) interface: u8,
    /// USB endpoint address for writing data to the printer (OUT endpoint)
    pub(crate) endpoint_out: u8,
    /// USB endpoint address for reading data from the printer (IN endpoint)
    pub(crate) endpoint_in: u8,
    /// Timeout for USB operations
    pub(crate) timeout: Duration,
}

impl UsbConnectionInfo {
    /// Create connection info from a printer model
    ///
    /// Returns the correct USB parameters (vendor ID, product ID, endpoints, etc.)
    /// for the specified printer model. This is the recommended way to create
    /// connection info for known printer models.
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::{connection::UsbConnectionInfo, printer::PrinterModel};
    /// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    /// assert_eq!(info.vendor_id, 0x04f9); // Brother vendor ID
    /// ```
    #[must_use]
    pub const fn from_model(model: PrinterModel) -> Self {
        Self {
            vendor_id: 0x04f9, // Brother
            product_id: model.product_id(),
            interface: 0,
            endpoint_out: 0x02,
            endpoint_in: 0x81,
            timeout: Duration::from_millis(5000),
        }
    }
}

/// USB connection to a Brother QL printer
pub struct UsbConnection {
    handle: DeviceHandle<Context>,
    interface: u8,
    timeout: Duration,
    endpoint_out: u8,
    endpoint_in: u8,
}

impl UsbConnection {
    /// Open a USB connection to a Brother QL printer
    ///
    /// Searches for a USB device matching the vendor and product IDs in the connection info,
    /// then claims the interface for exclusive access. The kernel driver is automatically
    /// detached and will be reattached when the connection is closed.
    ///
    /// # Errors
    /// Returns an error if:
    /// - No USB device is found with the specified vendor/product ID
    /// - The USB device cannot be opened
    /// - The interface cannot be claimed
    /// - USB configuration fails
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::{
    /// #     connection::{UsbConnection, UsbConnectionInfo},
    /// #     printer::PrinterModel,
    /// # };
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    /// let connection = UsbConnection::open(info)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open(info: UsbConnectionInfo) -> Result<Self, UsbError> {
        debug!("Opening USB Connection to the printer...");
        let context = Context::new()?;
        let device = Self::find_device(&context, info.vendor_id, info.product_id)?;
        let handle = device.open()?;

        // Auto-detach and reattach kernel driver when claiming/releasing
        handle.set_auto_detach_kernel_driver(true)?;
        if handle.kernel_driver_active(0)? {
            handle.detach_kernel_driver(0)?;
        }
        handle.set_active_configuration(1)?;
        // Claim the interface for exclusive access
        handle.claim_interface(info.interface)?;

        if let Err(e) = handle.set_alternate_setting(info.interface, 0) {
            // NOTE: Since we handle the failed alternate setting call we
            // propagate the original error instead of a possible cleanup one.
            let _ = handle.release_interface(info.interface);
            return Err(e.into());
        }

        debug!("Successfully established USB Connection!");
        Ok(Self {
            handle,
            interface: info.interface,
            timeout: info.timeout,
            endpoint_out: info.endpoint_out,
            endpoint_in: info.endpoint_in,
        })
    }

    /// Find a USB device with the specified vendor and product IDs
    fn find_device(
        context: &Context,
        vendor_id: u16,
        product_id: u16,
    ) -> Result<Device<Context>, UsbError> {
        let devices = context.devices()?;

        for device in devices.iter() {
            let descriptor = device.device_descriptor()?;
            if descriptor.vendor_id() == vendor_id && descriptor.product_id() == product_id {
                return Ok(device);
            }
        }

        Err(UsbError::DeviceNotFound {
            vendor_id,
            product_id,
        })
    }
}

// Implement the public connection interface
impl PrinterConnection for UsbConnection {}

// Implement the private connection interface
impl ConnectionImpl for UsbConnection {
    type Error = UsbError;

    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let bytes_written = self
            .handle
            .write_bulk(self.endpoint_out, data, self.timeout)?;
        if bytes_written != data.len() {
            return Err(UsbError::IncompleteWrite);
        }
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let bytes_read = self
            .handle
            .read_bulk(self.endpoint_in, buffer, self.timeout)?;
        Ok(bytes_read)
    }
}

impl Drop for UsbConnection {
    fn drop(&mut self) {
        // Release the interface when connection is dropped
        let _ = self.handle.release_interface(self.interface);
    }
}

use anyhow::{Result, anyhow};
use brother_ql::connection::{
    KernelConnection, PrinterConnection, UsbConnection, UsbConnectionInfo,
};

use crate::cli::PrinterSelection;

pub enum Connection {
    Usb(UsbConnection),
    Kernel(KernelConnection),
}

impl Connection {
    pub fn print(&mut self, job: brother_ql::printjob::PrintJob) -> Result<()> {
        match self {
            Connection::Usb(conn) => conn.print(job)?,
            Connection::Kernel(conn) => conn.print(job)?,
        };
        Ok(())
    }

    pub fn get_status(&mut self) -> Result<brother_ql::status::StatusInformation> {
        Ok(match self {
            Connection::Usb(conn) => conn.get_status()?,
            Connection::Kernel(conn) => conn.get_status()?,
        })
    }
}

pub fn create_connection(printer: PrinterSelection) -> Result<Connection> {
    match (printer.usb, printer.fd, printer.usb_auto_discover) {
        (Some(printer_model), _, _) => Ok(Connection::Usb(UsbConnection::open(
            UsbConnectionInfo::from_model(printer_model),
        )?)),
        (_, Some(path), _) => Ok(Connection::Kernel(KernelConnection::open(path)?)),
        (_, _, true) => {
            let conn_info = UsbConnectionInfo::discover()?
                .ok_or_else(|| anyhow!("Couldn't auto-discover any printers!"))?;
            Ok(Connection::Usb(UsbConnection::open(conn_info)?))
        }
        _ => unreachable!(),
    }
}

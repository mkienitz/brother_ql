use std::error::Error;

use brother_ql::{
    connection::{UsbConnection, UsbConnectionInfo},
    printer::PrinterModel,
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting to Brother QL-820NWB printer...");

    // Create connection info for QL-820NWB
    let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);

    // Open USB connection
    let mut connection = UsbConnection::open(info)?;
    println!("Connected successfully!");

    // Read status from printer
    println!("\nReading printer status...");
    let status = connection.get_status()?;

    // Full debug output
    println!("\n=== Full Status (Debug) ===");
    println!("{:#?}", status);

    Ok(())
}

# brother_ql

This is a crate to convert image data to the Raster Command binary data understood by the
Brother QL-8xx family of label printers.

## Features

- **📦 Compile to binary data** - Convert images to raster command bytes that can be sent to the printer via USB, network, or saved to files
- **🔌 Direct USB printing** - Print labels directly via USB connection with full status monitoring
- **📊 Status information** - Read detailed printer status including errors, media type, and operational phase
- **🎨 Two-color printing** - Support for red and black printing on compatible printer models
- **🏷️ Multiple media types** - Support for continuous and die-cut labels in various widths

## Supported Printers

Currently, the 8xx family of label printers is supported (QL-800, QL-810W, QL-820NWB). Other printers should be relatively easy to add.

**Note:** This crate is still work-in-progress and some bugs might still exist.

For more details, check the [official Raster Command Reference](https://download.brother.com/welcome/docp100278/cv_ql800_eng_raster_101.pdf).

## Examples

### Printing via USB connection

```rust
use brother_ql::{
    connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
    media::Media, printer::PrinterModel, printjob::PrintJob,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connection info for QL-820NWB
    let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    // Open USB connection
    let mut connection = UsbConnection::open(info)?;
    // Read status from printer
    let _status = connection.get_status()?;
    // Create a print job with more than one page
    let img = image::open("c62.png")?;
    let job = PrintJob::new(img, Media::C62)?.page_count(2);
    // These are the defaults for the other options:
    // .high_dpi(false)
    // .compressed(false)
    // .quality_priority(true)
    // .cut_behavior(CutBehavior::CutEach)?; // default for continuous media
    // Finally, print
    connection.print(job)?;
    Ok(())
}
```

### Compiling and saving a print job

```rust
use std::{fs::File, io::Write};
use brother_ql::{media::Media, printjob::PrintJob};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open("c62.png")?;
    let job = PrintJob::new(img, Media::C62)?;
    let data = job.compile();
    let mut file = File::create("c62mm.bin")?;
    file.write_all(&data)?;
    Ok(())
}
```

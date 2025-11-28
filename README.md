# brother_ql

This is a crate to convert image data to the Raster Command binary data understood by
Brother QL series label printers.

## Features

- **📦 Compile to binary data** - Convert images to raster command bytes that can be sent to the printer via USB, network, or saved to files
- **🔌 Direct USB printing** - Print labels directly via USB connection with full status monitoring (also supported via a kernel connection)
- **📊 Status information** - Read detailed printer status including errors, media type, and operational phase
- **🎨 Two-color printing** - Support for red and black printing on compatible printer models
- **🏷️ Multiple media types** - Support for continuous and die-cut labels in various widths

## Supported Printers

The following Brother QL label printers are supported:
- **5xx series**: QL-560, QL-570, QL-580N
- **6xx series**: QL-600, QL-650TD
- **7xx series**: QL-700 ✅, QL-710W, QL-720NW
- **8xx series**: QL-800, QL-810W, QL-820NWB ✅

**Legend:**
- ✅ = Tested and confirmed working
- No mark = Supported but not yet tested by contributors

**Help us test!** If you have one of the untested printer models, please try it out and let us know how it works! Feel free to:
- Open an issue to report successful testing
- Report any problems you encounter
- Contribute improvements to support additional models


**Note:** This crate is still work-in-progress and some bugs might still exist.

For more details, check the [official Raster Command Reference](https://download.brother.com/welcome/docp100278/cv_ql800_eng_raster_101.pdf) (this one is for the 8xx series).

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

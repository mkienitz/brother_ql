# Brother QL Rust Tools

Rust tools for Brother QL series label printers. This project provides both a [Rust library crate](https://crates.io/crates/brother_ql) for programmatic label printing and a [command-line tool](https://crates.io/crates/brother-label) for easy printing from the terminal.

## 📦 Components

### 📚 `brother_ql` - Rust Library

A Rust library to convert images to Brother QL raster command data and print labels directly via USB or kernel connections.

**Features:** Image to raster conversion, USB printing with status monitoring, two-color printing support, 28+ media types

**Links:**
- [README →](crates/brother_ql/README.md)
- [API Docs (docs.rs) →](https://docs.rs/brother_ql)
- [crates.io →](https://crates.io/crates/brother_ql)

**Quick example:**
```rust
use brother_ql::{connection::*, media::Media, printjob::PrintJob};

let mut conn = UsbConnection::open(UsbConnectionInfo::discover()?.unwrap())?;
let job = PrintJob::from_image(image::open("label.png")?, Media::C62)?;
conn.print(job)?;
```

### 🔧 `brother-label` - CLI Tool

A command-line application for printing labels to Brother QL printers. It's a minimal wrapper around the `brother_ql` library.

**Features:** Exposes almost all capabilities of the library crate

**Links:**
- [README →](crates/brother-label/README.md)
- [crates.io →](https://crates.io/crates/brother-label)

**Quick example:**
```bash
brother-label print label1.png label2.png --media d24 --usb-auto-discover --copies 4 --cut-behavior=no-cut
```

## 🖨️ Supported Printers

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

For more details, check the [official Raster Command Reference](https://download.brother.com/welcome/docp100278/cv_ql800_eng_raster_101.pdf) (this one is for the 8xx series).

## 🚀 Getting Started

**Want to print labels from the command line?**
→ See the [brother-label CLI documentation](crates/brother-label/README.md)

**Want to integrate printing into your Rust application?**
→ See the [brother_ql library documentation](crates/brother_ql/README.md)

## 💬 Contributing & Issues

This project is still new and hasn't been tested across all printer models and scenarios. If you encounter any problems, unexpected behavior, have successful test results to report, or have suggestions for improvements, please [report an issue on GitHub](https://github.com/mkienitz/brother_ql/issues/new/choose).

Your feedback helps make these tools better for everyone!

# brother-label

This is a [clap](https://github.com/clap-rs/clap)-based command-line application to use your Brother QL-series label printer.
It is based on this project's main library crate [brother_ql](https://crates.io/crates/brother_ql).

You can use the library instead if you want to integrate label printing inside your own application.

## 🚀 Quick Start

```bash
$ brother-label print mylabel.png --media c62 --usb-auto-discover
```

This will convert `mylabel.png` to raster command data, auto-discover the first connected USB printer and finally print your label.

**Note:**
- `c62` refers to continuous 62mm regular tape.
- The dimensions of the supplied images need to match the media type

For more information on label roll types and required image dimensions look at the [media type documentation](https://docs.rs/brother_ql/2.1.0/brother_ql/media/enum.Media.html)

## ✨ Features

- **🔌 USB Auto-Discovery** - Automatically find and connect to Brother QL printers with zero configuration
- **🖨️ Multiple Connection Types** - USB auto-discovery, specific model selection, or kernel driver (`/dev/usb/lp0`)
- **📝 Built-in Test Labels** - Generate test labels on-the-fly with Typst, no image files needed
- **📏 28 Media Types Supported** - Continuous tape (12-62mm), die-cut labels, and two-color printing
- **⚙️ Flexible Print Options** - Configure copies, cut behavior, quality, and high-dpi settings
- **📊 Status Monitoring** - Read and display printer status, media info, and error diagnostics

## 📖 Other Examples

### Print multiple images at once

```bash
$ brother-label print first.png second.png third.png --copies 5 --media d24 --cut-behavior=cut-at-end --usb ql820nwb
```

This will print 15 labels total (5 times the specified images sequence) onto circular 24mm die-cut labels.

**Note:**
- Whereas the Quick Start example used auto-discovery of connected USB printers, here we explicitly specify the printer model (QL 820-NWB)
- We instruct the printer to make a single cut at the end. This is the default for die-cut media types.

### Print using kernel device driver

```bash
$ brother-label print mylabel.png --media c62 --fd /dev/usb/lp0
```

This prints via the Linux kernel USB printer driver instead of direct USB communication.

**Note:**
The device path may vary (e.g., `/dev/usb/lp1`, `/dev/usb/lp2`) depending on your system and connected devices.

### Get the printer status

```bash
$ brother-label status --usb-auto-discover
```

This fetches the current printer status and prints it to your console.

### A quick print check using included test labels

```bash
$ brother-label print --media c62 --use-test-image --usb-auto-discover
```

This can be used to quickly check if the whole stack works as intended.
It dynamically creates compatible example labels using [typst](https://typst.app).

## 🖨️ Supported Printers

You can find the printer support status in the main project's [README](https://github.com/mkienitz/brother_ql?tab=readme-ov-file#supported-printers).

## 💬 Feedback & Issues

This project is still new and hasn't been tested across all printer models and scenarios. If you encounter any problems, unexpected behavior, or have suggestions for improvements, please [report an issue on GitHub](https://github.com/mkienitz/brother_ql/issues/new/choose).

Your feedback helps make this tool better for everyone!

